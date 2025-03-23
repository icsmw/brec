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
    Sized + PayloadEncode + PayloadHooks + PayloadEncodeReferred + PayloadSize + PayloadCrc + PayloadSignature
    // In code generator will be forced usage of WritePayloadWithHeaderTo
    + WriteMutTo
    // In code generator will be forced usage of WriteVectoredPayloadWithHeaderTo
    + WriteVectoredMutTo
{
}

/// Defines the outer container responsible for extracting a payload of a given `Inner` type.
pub trait PayloadDef<Inner: PayloadInnerDef>:
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
pub struct PacketDef<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    pub blocks: Vec<B>,

    /// Optional decoded payload.
    pub payload: Option<Inner>,

    /// Internal marker for payload definition type.
    _pi: PhantomData<P>,
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
    /// - Applies the `FilterByBlocks` rule
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
        rules: &RulesDef<B, BR, P, Inner>,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        R: std::io::Read + std::io::Seek,
        BR: BlockReferredDef<B>,
        Self: Sized,
    {
        let header = <PacketHeader as ReadFrom>::read(reader)?;
        let mut read = 0;
        let mut blocks = Vec::new();
        let mut packet_buffer = vec![0; header.size as usize];
        reader.read_exact(&mut packet_buffer)?;
        let blocks_len = header.blocks_len as usize;
        if blocks_len > 0 {
            loop {
                let blk =
                    <BR as ReadBlockFromSlice>::read_from_slice(&packet_buffer[read..], false)?;
                read += blk.size() as usize;
                blocks.push(blk);
                if read == blocks_len {
                    break;
                }
            }
        }
        let packet_size = header.size as usize;
        if !rules.filter_by_blocks(&blocks) {
            return Ok(LookInStatus::Denied(packet_size));
        }
        let pkg = if header.payload {
            let mut payload_buffer = &packet_buffer[blocks_len..];
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut payload_buffer)? {
                ReadStatus::Success(header) => {
                    let mut payload_buffer = &packet_buffer[blocks_len + header.size()..];
                    if !rules.filter_by_payload(payload_buffer) {
                        // PacketDef marked as ignored
                        return Ok(LookInStatus::Denied(packet_size));
                    }
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut payload_buffer,
                        &header,
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
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(needed as usize));
                }
            }
        } else {
            PacketDef::new(
                blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                None,
            )
        };
        if !rules.filter(&pkg) {
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
