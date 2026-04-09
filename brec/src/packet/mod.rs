mod header;
mod read;
mod reader;
mod referred;
mod rules;
mod write;

pub use header::*;
pub use reader::*;
pub use referred::*;
pub use rules::*;

use crate::*;

use std::marker::PhantomData;

/// Defines a fully parsed block type.
///
/// Required for reading, writing, size computation, and vectored I/O.
pub trait BlockReferredDef<B: BlockDef>: ReadBlockFromSlice + Size + Sized + Into<B> {}

/// Defines a block that refers to slices of existing memory (zero-copy).
///
/// This trait is commonly used for fast inspection or filtering without decoding full blocks.
/// It must support reading from a slice and conversion back to the owning `BlockDef` type.
pub trait BlockDef:
    ReadBlockFrom + ReadFrom + TryReadFrom + TryReadFromBuffered + WriteTo + WriteVectoredTo + Size
{
}

/// Defines the actual inner payload object used in packets.
///
/// Includes encoding, CRC, size, hooks, and full write logic (including headers).
pub trait PayloadInnerDef:
    Sized
    + PayloadSchema
    + PayloadEncode
    + PayloadHooks
    + PayloadEncodeReferred
    + PayloadSize
    + PayloadCrc
    + PayloadSignature
    // In code generator will be forced usage of WritePayloadWithHeaderTo
    + WriteMutTo
    // In code generator will be forced usage of WriteVectoredPayloadWithHeaderTo
    + WriteVectoredMutTo
{
}

/// Defines the outer container responsible for extracting a payload of a given `Inner` type.
pub trait PayloadDef<Inner: PayloadInnerDef>:
    ExtractPayloadFrom<Inner> + TryExtractPayloadFrom<Inner> + TryExtractPayloadFromBuffered<Inner>
{
}

/// Represents the result of a filtered packet inspection.
///
/// Used by `PacketDef::filtered()` to indicate whether a packet should be accepted,
/// denied, or postponed due to incomplete data.
pub enum LookInStatus<T> {
    /// The packet was accepted and returned, along with the number of bytes consumed.
    Accepted(usize, T),

    /// The packet was explicitly denied by the rule set.
    Denied(usize),

    /// Not enough data available to complete the operation. Indicates required amount.
    NotEnoughData(usize),
}

/// A complete parsed packet structure with block list and optional payload.
///
/// This structure is the result of reading a full `brec` packet, including:
/// - A vector of parsed blocks implementing `BlockDef`
/// - An optional payload implementing `PayloadInnerDef`
///
/// # Type Parameters
/// - `B`: Block type (fully parsed)
/// - `P`: Payload definition handler (extractor, size, etc.)
/// - `Inner`: Actual payload instance type
pub struct PacketDef<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    /// Fully parsed blocks stored in the packet.
    pub blocks: Vec<B>,

    /// Optional decoded payload.
    pub payload: Option<Inner>,

    /// Internal marker for payload definition type.
    _pi: PhantomData<P>,
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> PayloadSchema
    for PacketDef<B, P, Inner>
{
    type Context<'a> = <Inner as PayloadSchema>::Context<'a>;
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> PacketDef<B, P, Inner> {
    /// Creates a new packet from given blocks and optional payload.
    pub fn new(blocks: Vec<B>, payload: Option<Inner>) -> Self {
        Self {
            blocks,
            payload,
            _pi: PhantomData,
        }
    }

    /// Attempts to read and filter a packet from a stream using the provided rules.
    ///
    /// This function:
    /// - Reads the `PacketHeader`
    /// - Loads all blocks into memory
    /// - Applies the prefilter rule
    /// - Optionally parses and filters the payload
    /// - Returns the final result via `LookInStatus`
    ///
    /// # Limitations
    /// - Does **not** refill the stream buffer
    /// - Will fail if the entire packet is not already in the stream
    ///
    /// # Returns
    /// - `Accepted(bytes, packet)` - if all filters passed
    /// - `Denied(bytes)` - if blocked by rules
    /// - `NotEnoughData(bytes)` - if more input is needed
    ///
    /// # Errors
    /// - Propagates all decoding and parsing errors from blocks and payload
    pub fn filtered<R, BR>(
        reader: &mut R,
        rules: &RulesDef<B, BR, P, Inner>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        R: std::io::Read + std::io::Seek,
        BR: BlockReferredDef<B>,
        Self: Sized,
    {
        let header = <PacketHeader as ReadFrom>::read(reader)?;
        let mut read = 0usize;
        let mut blocks = Vec::new();
        let blocks_len = header.blocks_len as usize;
        if blocks_len > 0 {
            let mut blocks_buffer = vec![0; blocks_len];
            reader.read_exact(&mut blocks_buffer)?;
            loop {
                let blk =
                    <BR as ReadBlockFromSlice>::read_from_slice(&blocks_buffer[read..], false)?;
                read += blk.size() as usize;
                blocks.push(blk);
                if read == blocks_len {
                    break;
                }
            }
        }
        let packet_size = header.size as usize;
        if !rules.prefilter(&blocks) {
            if header.payload {
                let to_skip = packet_size.saturating_sub(blocks_len);
                if to_skip > 0 {
                    reader.seek(std::io::SeekFrom::Current(to_skip as i64))?;
                }
            }
            return Ok(LookInStatus::Denied(packet_size));
        }
        let pkg = if header.payload {
            let payload_header = <PayloadHeader as ReadFrom>::read(reader)?;
            let payload_body_len = payload_header.payload_len();
            if rules.has_payload_filter() {
                let mut payload_buffer = vec![0; payload_body_len];
                reader.read_exact(&mut payload_buffer)?;
                if !rules.filter_payload(&payload_buffer) {
                    return Ok(LookInStatus::Denied(packet_size));
                }
                let mut payload_reader = std::io::Cursor::new(payload_buffer);
                match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                    &mut payload_reader,
                    &payload_header,
                    ctx,
                ) {
                    Ok(ReadStatus::Success(payload)) => PacketDef::new(
                        blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                        Some(payload),
                    ),
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        return Err(Error::NotEnoughData(needed as usize));
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else {
                match <P as TryExtractPayloadFrom<Inner>>::try_read(reader, &payload_header, ctx) {
                    Ok(ReadStatus::Success(payload)) => PacketDef::new(
                        blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                        Some(payload),
                    ),
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        return Err(Error::NotEnoughData(needed as usize));
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        } else {
            PacketDef::new(
                blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                None,
            )
        };
        if !rules.filter_packet(&pkg) {
            // PacketDef marked as ignored
            Ok(LookInStatus::Denied(packet_size))
        } else {
            Ok(LookInStatus::Accepted(packet_size, pkg))
        }
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> Default for PacketDef<B, P, Inner> {
    /// Creates an empty `PacketDef` with no blocks and no payload.
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            payload: None,
            _pi: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Clone, Copy)]
    enum DecodeOutcome {
        Success,
        NotEnough(u64),
        ErrInvalidLength,
        ErrCrcDismatch,
    }

    #[derive(Clone, Copy)]
    struct DecodeCtx {
        buffered: DecodeOutcome,
        stream: DecodeOutcome,
    }

    #[derive(Clone)]
    struct TestPayload(u8);

    impl PayloadSchema for TestPayload {
        type Context<'a> = DecodeCtx;
    }

    impl PayloadHooks for TestPayload {}

    impl PayloadEncode for TestPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(vec![self.0])
        }
    }

    impl PayloadEncodeReferred for TestPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(Some(&[1_u8, 2_u8, 3_u8]))
        }
    }

    impl PayloadSignature for TestPayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(*b"TSTP")
        }
    }

    impl PayloadSize for TestPayload {}
    impl PayloadCrc for TestPayload {}

    impl WriteMutTo for TestPayload {
        fn write<T: std::io::Write>(
            &mut self,
            _: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<usize> {
            Ok(0)
        }

        fn write_all<T: std::io::Write>(
            &mut self,
            _: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl WriteVectoredMutTo for TestPayload {
        fn slices(&mut self, _: &mut Self::Context<'_>) -> std::io::Result<IoSlices<'_>> {
            Ok(IoSlices::default())
        }
    }

    impl PayloadInnerDef for TestPayload {}

    impl TryExtractPayloadFromBuffered<TestPayload> for TestPayload {
        fn try_read<B: std::io::BufRead>(
            _: &mut B,
            _: &PayloadHeader,
            ctx: &mut <TestPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<TestPayload>, Error> {
            match ctx.buffered {
                DecodeOutcome::Success => Ok(ReadStatus::Success(TestPayload(7))),
                DecodeOutcome::NotEnough(needed) => Ok(ReadStatus::NotEnoughData(needed)),
                DecodeOutcome::ErrInvalidLength => Err(Error::InvalidLength),
                DecodeOutcome::ErrCrcDismatch => Err(Error::CrcDismatch),
            }
        }
    }

    impl TryExtractPayloadFrom<TestPayload> for TestPayload {
        fn try_read<B: std::io::Read + std::io::Seek>(
            _: &mut B,
            _: &PayloadHeader,
            ctx: &mut <TestPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<TestPayload>, Error> {
            match ctx.stream {
                DecodeOutcome::Success => Ok(ReadStatus::Success(TestPayload(9))),
                DecodeOutcome::NotEnough(needed) => Ok(ReadStatus::NotEnoughData(needed)),
                DecodeOutcome::ErrInvalidLength => Err(Error::InvalidLength),
                DecodeOutcome::ErrCrcDismatch => Err(Error::CrcDismatch),
            }
        }
    }

    impl ExtractPayloadFrom<TestPayload> for TestPayload {
        fn read<B: std::io::Read>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <TestPayload as PayloadSchema>::Context<'_>,
        ) -> Result<TestPayload, Error> {
            panic!("unexpected ExtractPayloadFrom::read call in packet::mod tests")
        }
    }

    impl PayloadDef<TestPayload> for TestPayload {}

    struct TestBlock;
    struct TestBlockRef;

    impl Size for TestBlock {
        fn size(&self) -> u64 {
            panic!("unexpected TestBlock::size call in packet::mod tests")
        }
    }

    impl WriteTo for TestBlock {
        fn write<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<usize> {
            panic!("unexpected TestBlock::write call in packet::mod tests")
        }

        fn write_all<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<()> {
            panic!("unexpected TestBlock::write_all call in packet::mod tests")
        }
    }

    impl WriteVectoredTo for TestBlock {
        fn slices(&self) -> std::io::Result<IoSlices<'_>> {
            panic!("unexpected TestBlock::slices call in packet::mod tests")
        }
    }

    impl TryReadFromBuffered for TestBlock {
        fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
            panic!("unexpected TestBlock::try_read(buffered) call in packet::mod tests")
        }
    }

    impl TryReadFrom for TestBlock {
        fn try_read<T: std::io::Read + std::io::Seek>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
            panic!("unexpected TestBlock::try_read(stream) call in packet::mod tests")
        }
    }

    impl ReadFrom for TestBlock {
        fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error> {
            panic!("unexpected TestBlock::read call in packet::mod tests")
        }
    }

    impl ReadBlockFrom for TestBlock {
        fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error> {
            panic!("unexpected TestBlock::read(block) call in packet::mod tests")
        }
    }

    impl ReadBlockFromSlice for TestBlock {
        fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
        where
            Self: 'a + Sized,
        {
            panic!("unexpected TestBlock::read_from_slice call in packet::mod tests")
        }
    }

    impl BlockDef for TestBlock {}

    impl Size for TestBlockRef {
        fn size(&self) -> u64 {
            panic!("unexpected TestBlockRef::size call in packet::mod tests")
        }
    }

    impl ReadBlockFromSlice for TestBlockRef {
        fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
        where
            Self: 'a + Sized,
        {
            panic!("unexpected TestBlockRef::read_from_slice call in packet::mod tests")
        }
    }

    impl From<TestBlockRef> for TestBlock {
        fn from(_: TestBlockRef) -> TestBlock {
            TestBlock
        }
    }

    impl BlockReferredDef<TestBlock> for TestBlockRef {}

    fn packet_bytes_with_payload(payload: bool) -> Vec<u8> {
        let mut out = Vec::new();
        if payload {
            let body = vec![1_u8, 2, 3];
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&body);
            let payload_header = PayloadHeader {
                sig: ByteBlock::Len4(*b"TSTP"),
                crc: ByteBlock::Len4(hasher.finalize().to_le_bytes()),
                len: body.len() as u32,
            }
            .as_vec();
            let header = PacketHeader::from_lengths(
                0,
                (payload_header.len() + body.len()) as u64,
                true,
            );
            header.write_all(&mut out).expect("header");
            out.extend_from_slice(&payload_header);
            out.extend_from_slice(&body);
        } else {
            let header = PacketHeader::from_lengths(0, 0, false);
            header.write_all(&mut out).expect("header");
        }
        out
    }

    fn rules_with_payload_filter() -> RulesDef<TestBlock, TestBlockRef, TestPayload, TestPayload> {
        let mut rules = RulesDef::default();
        rules
            .add_rule(RuleDef::FilterPayload(RuleFnDef::Static(|_| true)))
            .expect("payload filter");
        rules
    }

    #[test]
    fn filtered_payload_buffered_success() {
        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let rules = rules_with_payload_filter();
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::Success,
            stream: DecodeOutcome::ErrInvalidLength,
        };
        let status = PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<_, TestBlockRef>(
            &mut reader,
            &rules,
            &mut ctx,
        )
        .expect("filtered");
        assert!(matches!(status, LookInStatus::Accepted(_, _)));
    }

    #[test]
    fn filtered_payload_buffered_not_enough() {
        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let rules = rules_with_payload_filter();
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::NotEnough(11),
            stream: DecodeOutcome::ErrInvalidLength,
        };
        let err = match PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<
            _,
            TestBlockRef,
        >(&mut reader, &rules, &mut ctx)
        {
            Ok(_) => panic!("must be not enough"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::NotEnoughData(11)));
    }

    #[test]
    fn filtered_payload_buffered_error() {
        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let rules = rules_with_payload_filter();
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::ErrInvalidLength,
            stream: DecodeOutcome::Success,
        };
        let err = match PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<
            _,
            TestBlockRef,
        >(&mut reader, &rules, &mut ctx)
        {
            Ok(_) => panic!("must propagate decode error"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::InvalidLength));
    }

    #[test]
    fn filtered_payload_stream_paths() {
        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let rules = RulesDef::<TestBlock, TestBlockRef, TestPayload, TestPayload>::default();
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::ErrCrcDismatch,
            stream: DecodeOutcome::Success,
        };
        let status = PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<_, TestBlockRef>(
            &mut reader,
            &rules,
            &mut ctx,
        )
        .expect("stream success");
        assert!(matches!(status, LookInStatus::Accepted(_, _)));

        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::ErrCrcDismatch,
            stream: DecodeOutcome::NotEnough(13),
        };
        let err = match PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<
            _,
            TestBlockRef,
        >(&mut reader, &rules, &mut ctx)
        {
            Ok(_) => panic!("stream not enough"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::NotEnoughData(13)));

        let mut reader = Cursor::new(packet_bytes_with_payload(true));
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::ErrCrcDismatch,
            stream: DecodeOutcome::ErrInvalidLength,
        };
        let err = match PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<
            _,
            TestBlockRef,
        >(&mut reader, &rules, &mut ctx)
        {
            Ok(_) => panic!("stream error"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::InvalidLength));
    }

    #[test]
    fn filtered_packet_filter_can_deny_packet() {
        let mut reader = Cursor::new(packet_bytes_with_payload(false));
        let mut rules = RulesDef::<TestBlock, TestBlockRef, TestPayload, TestPayload>::default();
        rules
            .add_rule(RuleDef::FilterPacket(RuleFnDef::Static(|_| false)))
            .expect("packet filter");
        let mut ctx = DecodeCtx {
            buffered: DecodeOutcome::Success,
            stream: DecodeOutcome::Success,
        };
        let status = PacketDef::<TestBlock, TestPayload, TestPayload>::filtered::<_, TestBlockRef>(
            &mut reader,
            &rules,
            &mut ctx,
        )
        .expect("filtered");
        assert!(matches!(status, LookInStatus::Denied(_)));
    }
}
