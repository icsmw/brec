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
pub trait PayloadInnerDef<O: Default = ()>:
    Sized
    + PayloadEncode<O>
    + PayloadHooks
    + PayloadEncodeReferred<O>
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
pub trait PayloadDef<O: Default, Inner: PayloadInnerDef<O>>:
    ExtractPayloadFrom<Inner>
    + TryExtractPayloadFrom<Inner>
    + TryExtractPayloadFromBuffered<Inner>
    + PayloadSize
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
pub struct PacketDef<O: Default, B: BlockDef, P: PayloadDef<O, Inner>, Inner: PayloadInnerDef<O>>
{
    pub blocks: Vec<B>,

    /// Optional decoded payload.
    pub payload: Option<Inner>,

    /// Internal marker for payload definition type.
    _pi: PhantomData<P>,
    _po: PhantomData<fn() -> O>,
}

impl<O: Default, B: BlockDef, P: PayloadDef<O, Inner>, Inner: PayloadInnerDef<O>>
    PacketDef<O, B, P, Inner>
{
    /// Creates a new packet from given blocks and optional payload.
    pub fn new(blocks: Vec<B>, payload: Option<Inner>) -> Self {
        Self {
            blocks,
            payload,
            _pi: PhantomData,
            _po: PhantomData,
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
    /// - `Accepted(bytes, packet)` — if all filters passed
    /// - `Denied(bytes)` — if blocked by rules
    /// - `NotEnoughData(bytes)` — if more input is needed
    ///
    /// # Errors
    /// - Propagates all decoding and parsing errors from blocks and payload
    pub fn filtered<R, BR>(
        reader: &mut R,
        rules: &RulesDef<O, B, BR, P, Inner>,
    ) -> Result<LookInStatus<PacketDef<O, B, P, Inner>>, Error>
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
                let blk = <BR as ReadBlockFromSlice>::read_from_slice(&blocks_buffer[read..], false)?;
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
                match <P as TryExtractPayloadFrom<Inner>>::try_read(reader, &payload_header) {
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

impl<O: Default, B: BlockDef, P: PayloadDef<O, Inner>, Inner: PayloadInnerDef<O>> Default
    for PacketDef<O, B, P, Inner>
{
    /// Creates an empty `PacketDef` with no blocks and no payload.
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            payload: None,
            _pi: PhantomData,
            _po: PhantomData,
        }
    }
}
