use crate::payload::EncodedPayload;
use crate::*;

/// Implements mutable stream writing for a full `PacketDef`, including header, blocks, and payload.
///
/// This trait supports writing packets to a stream with partial write handling
/// (`write`) and guaranteed complete writes (`write_all`).
///
/// # Behavior
/// - The `PacketHeader` is constructed on the fly based on the current blocks and payload.
/// - `write()` may return early if only part of the data was written.
/// - `write_all()` retries until all data is successfully written.
///
/// # Errors
/// Returns any I/O error or encoding failure from `BlockDef`, `PayloadDef`, or `PayloadHeader`.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteMutTo
    for PacketDef<B, P, Inner>
{
    /// Writes the packet to a stream, allowing for partial write detection.
    ///
    /// # Returns
    /// Total number of bytes written. May be less than full packet size if the underlying
    /// writer cannot accept all data in one go.
    ///
    /// Use `write_all()` if full delivery is required.
    fn write<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
        let mut total = header.write(buf)?;
        if total < PacketHeader::SIZE as usize {
            return Ok(total);
        }
        for blk in self.blocks.iter() {
            let size = blk.size() as usize;
            let written = blk.write(buf)?;
            if written < size {
                return Ok(total + written);
            }
            total += written;
        }
        if let Some((payload_header, payload_body)) = prepared_payload.as_ref() {
            let payload_header = payload_header.as_vec();
            let written = buf.write(&payload_header)?;
            if written < payload_header.len() {
                return Ok(total + written);
            }
            total += written;

            let written = buf.write(payload_body.as_slice())?;
            if written < payload_body.len() {
                return Ok(total + written);
            }
            total += written;
        }
        Ok(total)
    }

    /// Writes the entire packet to the stream, retrying until all parts are written.
    ///
    /// This includes:
    /// - the computed `PacketHeader`
    /// - each individual block
    /// - optional payload
    fn write_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
        header.write_all(buf)?;
        for blk in self.blocks.iter() {
            blk.write_all(buf)?;
        }
        if let Some((payload_header, payload_body)) = prepared_payload.as_ref() {
            buf.write_all(&payload_header.as_vec())?;
            buf.write_all(payload_body.as_slice())?;
        }
        Ok(())
    }
}

/// Implements vectored I/O writing for `PacketDef` using `IoSlices`.
///
/// This trait allows the entire packet to be described as a collection of contiguous slices,
/// which can then be written efficiently using `write_vectored()` or `write_vectored_all()`.
///
/// # Behavior
/// - Builds a dynamic `IoSlices` buffer including header, blocks, and optional payload.
/// - Encodes the header into a temporary buffer and adds it as the first slice.
/// - Calls `slices()` on each block and the payload (if present).
///
/// # Errors
/// Returns an error if header construction, encoding, or slice generation fails.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteVectoredMutTo
    for PacketDef<B, P, Inner>
{
    /// Returns an `IoSlices` collection representing the full serialized packet.
    ///
    /// This includes:
    /// - Serialized header (as buffered bytes)
    /// - Serialized block slices
    /// - Serialized payload slices (if any)
    ///
    /// # Returns
    /// A ready-to-write `IoSlices` that can be passed to `write_vectored`.
    fn slices(
        &mut self,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<IoSlices<'_>> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
        let mut slices = IoSlices::default();
        let mut header_bytes: Vec<u8> = Vec::new();
        header.write_all(&mut header_bytes)?;
        slices.add_buffered(header_bytes);
        for blk in self.blocks.iter() {
            slices.append(blk.slices()?);
        }
        if let Some((payload_header, payload_body)) = prepared_payload {
            slices.add_buffered(payload_header.as_vec());
            match payload_body {
                EncodedPayload::Borrowed(bytes) => slices.add_slice(bytes),
                EncodedPayload::Owned(bytes) => slices.add_buffered(bytes),
            }
        }
        Ok(slices)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ByteBlock, DefaultPayloadContext, Error, ExtractPayloadFrom, IoSlices, PacketDef,
        PacketHeader, PayloadCrc, PayloadDef, PayloadEncode, PayloadEncodeReferred, PayloadHeader,
        PayloadHooks, PayloadInnerDef, PayloadSchema, PayloadSignature, PayloadSize, ReadFrom,
        ReadStatus, TryExtractPayloadFrom, TryExtractPayloadFromBuffered, TryReadFrom,
        TryReadFromBuffered, WriteMutTo, WriteTo, WriteVectoredMutTo, WriteVectoredTo,
    };
    use std::io::{Cursor, Read, Write};

    struct OkBlock(Vec<u8>);

    impl crate::Size for OkBlock {
        fn size(&self) -> u64 {
            self.0.len() as u64
        }
    }

    impl WriteTo for OkBlock {
        fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
            buf.write(self.0.as_slice())
        }
        fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
            buf.write_all(self.0.as_slice())
        }
    }

    impl WriteVectoredTo for OkBlock {
        fn slices(&self) -> std::io::Result<IoSlices<'_>> {
            let mut slices = IoSlices::default();
            slices.add_slice(self.0.as_slice());
            Ok(slices)
        }
    }

    // These read/extract methods are required by `BlockDef`/`PayloadDef` bounds for
    // packet generics in tests. They are intentional stub implementations that return
    // explicit errors, and we call them in a dedicated test so coverage metrics stay honest.
    impl TryReadFromBuffered for OkBlock {
        fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }
    impl TryReadFrom for OkBlock {
        fn try_read<T: std::io::Read + std::io::Seek>(
            _: &mut T,
        ) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadFrom for OkBlock {
        fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadBlockFrom for OkBlock {
        fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadBlockFromSlice for OkBlock {
        fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
        where
            Self: 'a + Sized,
        {
            Err(Error::Test)
        }
    }
    impl crate::BlockDef for OkBlock {}

    struct ErrBlock;

    impl crate::Size for ErrBlock {
        fn size(&self) -> u64 {
            1
        }
    }
    impl WriteTo for ErrBlock {
        fn write<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<usize> {
            Err(std::io::Error::other("err block write"))
        }
        fn write_all<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<()> {
            Err(std::io::Error::other("err block write_all"))
        }
    }
    impl WriteVectoredTo for ErrBlock {
        fn slices(&self) -> std::io::Result<IoSlices<'_>> {
            Err(std::io::Error::other("err block slices"))
        }
    }
    impl TryReadFromBuffered for ErrBlock {
        fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }
    impl TryReadFrom for ErrBlock {
        fn try_read<T: std::io::Read + std::io::Seek>(
            _: &mut T,
        ) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadFrom for ErrBlock {
        fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadBlockFrom for ErrBlock {
        fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }
    impl crate::ReadBlockFromSlice for ErrBlock {
        fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
        where
            Self: 'a + Sized,
        {
            Err(Error::Test)
        }
    }
    impl crate::BlockDef for ErrBlock {}

    #[derive(Clone)]
    struct LocalPayload(Vec<u8>);

    impl PayloadSchema for LocalPayload {
        type Context<'a> = DefaultPayloadContext;
    }
    impl PayloadHooks for LocalPayload {}
    impl PayloadEncode for LocalPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.0.clone())
        }
    }
    impl PayloadEncodeReferred for LocalPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(Some(self.0.as_slice()))
        }
    }
    impl PayloadSignature for LocalPayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(*b"DATA")
        }
    }
    impl PayloadSize for LocalPayload {}
    impl PayloadCrc for LocalPayload {}
    impl WriteMutTo for LocalPayload {
        fn write<T: std::io::Write>(
            &mut self,
            buf: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<usize> {
            buf.write(self.0.as_slice())
        }
        fn write_all<T: std::io::Write>(
            &mut self,
            buf: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<()> {
            buf.write_all(self.0.as_slice())
        }
    }
    impl WriteVectoredMutTo for LocalPayload {
        fn slices(&mut self, _: &mut Self::Context<'_>) -> std::io::Result<IoSlices<'_>> {
            let mut slices = IoSlices::default();
            slices.add_slice(self.0.as_slice());
            Ok(slices)
        }
    }
    impl PayloadInnerDef for LocalPayload {}
    impl TryExtractPayloadFromBuffered<LocalPayload> for LocalPayload {
        fn try_read<B: std::io::BufRead>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <LocalPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<LocalPayload>, Error> {
            Err(Error::Test)
        }
    }
    impl TryExtractPayloadFrom<LocalPayload> for LocalPayload {
        fn try_read<B: std::io::Read + std::io::Seek>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <LocalPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<LocalPayload>, Error> {
            Err(Error::Test)
        }
    }
    impl ExtractPayloadFrom<LocalPayload> for LocalPayload {
        fn read<B: std::io::Read>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <LocalPayload as PayloadSchema>::Context<'_>,
        ) -> Result<LocalPayload, Error> {
            Err(Error::Test)
        }
    }
    impl PayloadDef<LocalPayload> for LocalPayload {}

    #[derive(Clone)]
    struct OwnedPayload(Vec<u8>);

    impl PayloadSchema for OwnedPayload {
        type Context<'a> = DefaultPayloadContext;
    }
    impl PayloadHooks for OwnedPayload {}
    impl PayloadEncode for OwnedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.0.clone())
        }
    }
    impl PayloadEncodeReferred for OwnedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(None)
        }
    }
    impl PayloadSignature for OwnedPayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(*b"OWND")
        }
    }
    impl PayloadSize for OwnedPayload {}
    impl PayloadCrc for OwnedPayload {}
    impl WriteMutTo for OwnedPayload {
        fn write<T: std::io::Write>(
            &mut self,
            buf: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<usize> {
            buf.write(self.0.as_slice())
        }
        fn write_all<T: std::io::Write>(
            &mut self,
            buf: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<()> {
            buf.write_all(self.0.as_slice())
        }
    }
    impl WriteVectoredMutTo for OwnedPayload {
        fn slices(&mut self, _: &mut Self::Context<'_>) -> std::io::Result<IoSlices<'_>> {
            let mut slices = IoSlices::default();
            slices.add_slice(self.0.as_slice());
            Ok(slices)
        }
    }
    impl PayloadInnerDef for OwnedPayload {}
    impl TryExtractPayloadFromBuffered<OwnedPayload> for OwnedPayload {
        fn try_read<B: std::io::BufRead>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <OwnedPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<OwnedPayload>, Error> {
            Err(Error::Test)
        }
    }
    impl TryExtractPayloadFrom<OwnedPayload> for OwnedPayload {
        fn try_read<B: std::io::Read + std::io::Seek>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <OwnedPayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<OwnedPayload>, Error> {
            Err(Error::Test)
        }
    }
    impl ExtractPayloadFrom<OwnedPayload> for OwnedPayload {
        fn read<B: std::io::Read>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <OwnedPayload as PayloadSchema>::Context<'_>,
        ) -> Result<OwnedPayload, Error> {
            Err(Error::Test)
        }
    }
    impl PayloadDef<OwnedPayload> for OwnedPayload {}

    struct LimitWriter {
        max: usize,
        out: Vec<u8>,
    }

    impl Write for LimitWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let n = buf.len().min(self.max.max(1));
            self.out.extend_from_slice(&buf[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct ScriptedWriter {
        limits: Vec<usize>,
        call: usize,
        out: Vec<u8>,
    }

    impl Write for ScriptedWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let limit = self
                .limits
                .get(self.call)
                .copied()
                .unwrap_or(usize::MAX)
                .max(1);
            self.call += 1;
            let n = buf.len().min(limit);
            self.out.extend_from_slice(&buf[..n]);
            Ok(n)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_and_write_all_work_for_empty_packet() {
        let mut packet = PacketDef::<OkBlock, LocalPayload, LocalPayload>::default();
        let mut out = Vec::new();
        let written = packet
            .write(&mut out, &mut DefaultPayloadContext::default())
            .expect("write");
        assert_eq!(written, PacketHeader::SIZE as usize);
        assert_eq!(out.len(), PacketHeader::SIZE as usize);

        let mut out_all = Vec::new();
        packet
            .write_all(&mut out_all, &mut DefaultPayloadContext::default())
            .expect("write_all");
        assert_eq!(out_all.len(), PacketHeader::SIZE as usize);
        assert_eq!(out, out_all);
    }

    #[test]
    fn write_returns_partial_when_writer_is_limited() {
        let mut packet = PacketDef::<OkBlock, LocalPayload, LocalPayload>::default();
        let mut writer = LimitWriter {
            max: 3,
            out: Vec::new(),
        };
        let written = packet
            .write(&mut writer, &mut DefaultPayloadContext::default())
            .expect("partial write");
        assert_eq!(written, 3);
        assert_eq!(writer.out.len(), 3);
        writer.flush().expect("flush");
    }

    #[test]
    fn write_with_block_and_payload_matches_write_all() {
        let payload = LocalPayload(vec![9, 8, 7, 6]);
        let mut packet = PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(
            vec![OkBlock(vec![1, 2, 3])],
            Some(payload.clone()),
        );
        let mut out = Vec::new();
        let written = packet
            .write(&mut out, &mut DefaultPayloadContext::default())
            .expect("write");

        let mut out_all = Vec::new();
        let mut packet_all = PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(
            vec![OkBlock(vec![1, 2, 3])],
            Some(payload),
        );
        packet_all
            .write_all(&mut out_all, &mut DefaultPayloadContext::default())
            .expect("write_all");

        assert_eq!(written, out.len());
        assert_eq!(out, out_all);
    }

    #[test]
    fn write_returns_partial_when_block_write_is_short() {
        let mut packet = PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(
            vec![OkBlock(vec![1, 2, 3])],
            None,
        );
        let mut writer = ScriptedWriter {
            limits: vec![PacketHeader::SIZE as usize, 1],
            call: 0,
            out: Vec::new(),
        };

        let written = packet
            .write(&mut writer, &mut DefaultPayloadContext::default())
            .expect("write");
        assert_eq!(written, PacketHeader::SIZE as usize + 1);
    }

    #[test]
    fn write_returns_partial_when_payload_header_is_short() {
        let payload = LocalPayload(vec![10, 20, 30, 40]);
        let payload_header_len =
            PayloadHeader::new(&payload, &mut DefaultPayloadContext::default())
                .expect("payload header")
                .size();
        let mut packet =
            PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(vec![], Some(payload));
        let mut writer = ScriptedWriter {
            limits: vec![PacketHeader::SIZE as usize, payload_header_len - 1],
            call: 0,
            out: Vec::new(),
        };

        let written = packet
            .write(&mut writer, &mut DefaultPayloadContext::default())
            .expect("write");
        assert_eq!(
            written,
            PacketHeader::SIZE as usize + payload_header_len - 1
        );
    }

    #[test]
    fn write_returns_partial_when_payload_body_is_short() {
        let payload = LocalPayload(vec![10, 20, 30, 40]);
        let payload_header_len =
            PayloadHeader::new(&payload, &mut DefaultPayloadContext::default())
                .expect("payload header")
                .size();
        let mut packet =
            PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(vec![], Some(payload.clone()));
        let mut writer = ScriptedWriter {
            limits: vec![
                PacketHeader::SIZE as usize,
                payload_header_len,
                payload.0.len() - 1,
            ],
            call: 0,
            out: Vec::new(),
        };

        let written = packet
            .write(&mut writer, &mut DefaultPayloadContext::default())
            .expect("write");
        assert_eq!(
            written,
            PacketHeader::SIZE as usize + payload_header_len + payload.0.len() - 1
        );
    }

    #[test]
    fn write_and_slices_propagate_block_errors() {
        let mut packet =
            PacketDef::<ErrBlock, LocalPayload, LocalPayload>::new(vec![ErrBlock], None);
        let err = packet
            .write(&mut Vec::new(), &mut DefaultPayloadContext::default())
            .expect_err("block write must fail");
        assert_eq!(err.kind(), std::io::ErrorKind::Other);

        let err = match packet.slices(&mut DefaultPayloadContext::default()) {
            Ok(_) => panic!("block slices must fail"),
            Err(err) => err,
        };
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn write_all_propagates_block_error() {
        let mut packet =
            PacketDef::<ErrBlock, LocalPayload, LocalPayload>::new(vec![ErrBlock], None);
        let err = packet
            .write_all(&mut Vec::new(), &mut DefaultPayloadContext::default())
            .expect_err("block write_all must fail");
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn write_all_and_slices_include_payload() {
        let payload = LocalPayload(vec![10, 20, 30, 40]);
        let mut packet = PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(
            vec![OkBlock(vec![1, 2])],
            Some(payload.clone()),
        );

        let mut out = Vec::new();
        packet
            .write_all(&mut out, &mut DefaultPayloadContext::default())
            .expect("write_all");

        let mut cursor = Cursor::new(out.as_slice());
        let packet_header = PacketHeader::read(&mut cursor).expect("packet header read");
        assert_eq!(packet_header.blocks_len, 2);
        assert!(packet_header.payload);

        let mut block = [0_u8; 2];
        cursor.read_exact(&mut block).expect("read block");
        assert_eq!(block, [1, 2]);

        let payload_header = PayloadHeader::read(&mut cursor).expect("payload header read");
        assert_eq!(payload_header.payload_len(), 4);
        let mut payload_body = [0_u8; 4];
        cursor
            .read_exact(&mut payload_body)
            .expect("read payload body");
        assert_eq!(payload_body, [10, 20, 30, 40]);

        let mut packet2 = PacketDef::<OkBlock, LocalPayload, LocalPayload>::new(
            vec![OkBlock(vec![1, 2])],
            Some(payload),
        );
        let slices = packet2
            .slices(&mut DefaultPayloadContext::default())
            .expect("slices");
        let mut vectored = Vec::new();
        for s in slices.get() {
            vectored.extend_from_slice(&s);
        }
        assert_eq!(vectored, out);
    }

    #[test]
    fn slices_include_owned_payload_when_referred_is_missing() {
        let payload = OwnedPayload(vec![11, 22, 33, 44]);
        let mut packet = PacketDef::<OkBlock, OwnedPayload, OwnedPayload>::new(
            vec![OkBlock(vec![1, 2])],
            Some(payload.clone()),
        );
        let slices = packet
            .slices(&mut DefaultPayloadContext::default())
            .expect("slices");
        let mut vectored = Vec::new();
        for s in slices.get() {
            vectored.extend_from_slice(&s);
        }

        let mut packet_all = PacketDef::<OkBlock, OwnedPayload, OwnedPayload>::new(
            vec![OkBlock(vec![1, 2])],
            Some(payload),
        );
        let mut out_all = Vec::new();
        packet_all
            .write_all(&mut out_all, &mut DefaultPayloadContext::default())
            .expect("write_all");

        assert_eq!(vectored, out_all);
    }

    #[test]
    fn trait_required_stub_methods_return_explicit_errors() {
        let mut buffered = Cursor::new(Vec::<u8>::new());
        let mut stream = Cursor::new(Vec::<u8>::new());
        let payload_header = PayloadHeader {
            sig: ByteBlock::Len4([0, 0, 0, 0]),
            crc: ByteBlock::Len4([0, 0, 0, 0]),
            len: 0,
        };

        assert!(matches!(
            <OkBlock as TryReadFromBuffered>::try_read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OkBlock as TryReadFrom>::try_read(&mut stream),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OkBlock as crate::ReadFrom>::read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OkBlock as crate::ReadBlockFrom>::read(&mut buffered, false),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OkBlock as crate::ReadBlockFromSlice>::read_from_slice(&[], false),
            Err(Error::Test)
        ));

        assert!(matches!(
            <ErrBlock as TryReadFromBuffered>::try_read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <ErrBlock as TryReadFrom>::try_read(&mut stream),
            Err(Error::Test)
        ));
        assert!(matches!(
            <ErrBlock as crate::ReadFrom>::read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <ErrBlock as crate::ReadBlockFrom>::read(&mut buffered, false),
            Err(Error::Test)
        ));
        assert!(matches!(
            <ErrBlock as crate::ReadBlockFromSlice>::read_from_slice(&[], false),
            Err(Error::Test)
        ));

        let mut local_payload = LocalPayload(vec![1, 2, 3]);
        assert!(matches!(
            <LocalPayload as TryExtractPayloadFromBuffered<LocalPayload>>::try_read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <LocalPayload as TryExtractPayloadFrom<LocalPayload>>::try_read(
                &mut stream,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <LocalPayload as ExtractPayloadFrom<LocalPayload>>::read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert_eq!(
            <LocalPayload as WriteMutTo>::write(&mut local_payload, &mut Vec::new(), &mut ())
                .expect("local payload write"),
            3
        );

        let mut owned_payload = OwnedPayload(vec![4, 5, 6]);
        assert!(matches!(
            <OwnedPayload as TryExtractPayloadFromBuffered<OwnedPayload>>::try_read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OwnedPayload as TryExtractPayloadFrom<OwnedPayload>>::try_read(
                &mut stream,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <OwnedPayload as ExtractPayloadFrom<OwnedPayload>>::read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert_eq!(
            <OwnedPayload as WriteMutTo>::write(&mut owned_payload, &mut Vec::new(), &mut ())
                .expect("owned payload write"),
            3
        );
    }
}
