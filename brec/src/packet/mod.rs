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

pub trait BlockReferredDef<B: BlockDef>: ReadBlockFromSlice + Size + Sized + Into<B> {}

pub trait BlockDef:
    ReadBlockFrom + ReadFrom + TryReadFrom + TryReadFromBuffered + WriteTo + WriteVectoredTo + Size
{
}

pub trait PayloadInnerDef:
    Sized + PayloadEncode + PayloadHooks + PayloadEncodeReferred + PayloadSize + PayloadCrc + PayloadSignature
    // In code generator will be forced usage of WritePayloadWithHeaderTo
    + WriteMutTo
    // In code generator will be forced usage of WriteVectoredPayloadWithHeaderTo
    + WriteVectoredMutTo
{
}

pub trait PayloadDef<Inner: PayloadInnerDef>:
    ExtractPayloadFrom<Inner>
    + TryExtractPayloadFrom<Inner>
    + TryExtractPayloadFromBuffered<Inner>
    + PayloadSize
{
}

pub enum LookInStatus<T> {
    Accepted(usize, T),
    Denied(usize),
    NotEnoughData(usize),
}

pub struct PacketDef<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    pub blocks: Vec<B>,
    pub payload: Option<Inner>,
    _pi: PhantomData<P>,
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> PacketDef<B, P, Inner> {
    pub fn new(blocks: Vec<B>, payload: Option<Inner>) -> Self {
        Self {
            blocks,
            payload,
            _pi: PhantomData,
        }
    }
    /// This method cannot refill buffer; it will fail if there are not enough data
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
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            payload: None,
            _pi: PhantomData,
        }
    }
}
