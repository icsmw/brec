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
    pub fn filtered_by_blocks<R, BR, F>(
        reader: &mut R,
        mut filter: F,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        R: std::io::Read + std::io::Seek,
        BR: BlockReferredDef<B>,
        F: FnMut(&[BR]) -> bool,
        Self: Sized,
    {
        let header = match <PacketHeader as TryReadFrom>::try_read(reader)? {
            ReadStatus::NotEnoughData(needed) => return Err(Error::NotEnoughData(needed as usize)),
            ReadStatus::Success(header) => header,
        };
        let start_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))? - start_pos;
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < header.size {
            return Err(Error::NotEnoughData((header.size - len) as usize));
        }
        let mut read = 0;
        let mut blocks = Vec::new();
        let mut buffer = vec![0; header.size as usize];
        reader.read_exact(&mut buffer)?;
        if header.blocks_len > 0 {
            loop {
                let blk = <BR as ReadBlockFromSlice>::read_from_slice(&buffer[read..], false)?;
                read += blk.size() as usize;
                blocks.push(blk);
                if read == header.blocks_len as usize {
                    break;
                }
            }
        }
        if !filter(&blocks) {
            return Ok(LookInStatus::Denied(header.size as usize));
        }
        let mut pkg: PacketDef<B, P, Inner> = PacketDef {
            blocks: blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
            payload: None,
            _pi: PhantomData,
        };
        if header.payload {
            let mut reader = std::io::BufReader::new(&buffer[read..]);
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut reader,
                        &header,
                    ) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
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
        }
        Ok(LookInStatus::Accepted(header.size as usize, pkg))
    }
    pub fn filtered_by_payload<R, F>(
        reader: &mut R,
        mut filter: F,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        R: std::io::Read + std::io::Seek,
        F: FnMut(&[u8]) -> bool,
        Self: Sized,
    {
        let header = match <PacketHeader as TryReadFrom>::try_read(reader)? {
            ReadStatus::NotEnoughData(needed) => return Err(Error::NotEnoughData(needed as usize)),
            ReadStatus::Success(header) => header,
        };
        let start_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))? - start_pos;
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < header.size {
            return Err(Error::NotEnoughData((header.size - len) as usize));
        }
        let mut read = 0;
        let mut pkg: PacketDef<B, P, Inner> = PacketDef::default();
        if header.blocks_len > 0 {
            loop {
                match <B as TryReadFrom>::try_read(reader) {
                    Ok(ReadStatus::Success(blk)) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        return Err(Error::NotEnoughData(needed as usize));
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }
        if header.payload {
            let mut buffer = vec![0; (header.size - header.blocks_len) as usize];
            reader.read_exact(&mut buffer)?;
            if !filter(&buffer) {
                return Ok(LookInStatus::Denied(header.size as usize));
            }
            let mut reader = std::io::BufReader::new(&buffer[..]);
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut reader,
                        &header,
                    ) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
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
        }
        Ok(LookInStatus::Accepted(header.size as usize, pkg))
    }
    pub fn filtered<R, BR, FB, FP>(
        reader: &mut R,
        mut filter_by_blocks: FB,
        mut filter_by_payload: FP,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        R: std::io::Read + std::io::Seek,
        BR: BlockReferredDef<B>,
        FB: FnMut(&[BR]) -> bool,
        FP: FnMut(&[u8]) -> bool,
        Self: Sized,
    {
        let header = match <PacketHeader as TryReadFrom>::try_read(reader)? {
            ReadStatus::NotEnoughData(needed) => return Err(Error::NotEnoughData(needed as usize)),
            ReadStatus::Success(header) => header,
        };
        let start_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))? - start_pos;
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < header.size {
            return Err(Error::NotEnoughData((header.size - len) as usize));
        }
        let mut read = 0;
        let mut blocks = Vec::new();
        let mut buffer = vec![0; header.size as usize];
        reader.read_exact(&mut buffer)?;
        if header.blocks_len > 0 {
            loop {
                let blk = <BR as ReadBlockFromSlice>::read_from_slice(&buffer[read..], false)?;
                read += blk.size() as usize;
                blocks.push(blk);
                if read == header.blocks_len as usize {
                    break;
                }
            }
        }
        if !filter_by_blocks(&blocks) {
            return Ok(LookInStatus::Denied(header.size as usize));
        }
        let mut pkg: PacketDef<B, P, Inner> = PacketDef {
            blocks: blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
            payload: None,
            _pi: PhantomData,
        };
        if header.payload {
            let mut buffer = vec![0; (header.size - header.blocks_len) as usize];
            reader.read_exact(&mut buffer)?;
            if !filter_by_payload(&buffer) {
                return Ok(LookInStatus::Denied(header.size as usize));
            }
            let mut reader = std::io::BufReader::new(&buffer[..]);
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut reader,
                        &header,
                    ) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
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
        }
        Ok(LookInStatus::Accepted(header.size as usize, pkg))
    }
    pub fn look_in<BR, F>(
        bytes: &[u8],
        chk: F,
    ) -> Result<LookInStatus<PacketDef<B, P, Inner>>, Error>
    where
        BR: BlockReferredDef<B>,
        F: Fn(&[BR]) -> bool,
        Self: Sized,
    {
        let available = bytes.len() as u64;
        if available < PacketHeader::ssize() {
            return Ok(LookInStatus::NotEnoughData(
                (PacketHeader::ssize() - available) as usize,
            ));
        }
        let header = PacketHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(LookInStatus::NotEnoughData(
                (header.size - available) as usize,
            ));
        }
        let mut blocks = Vec::new();
        let mut read = PacketHeader::ssize() as usize;
        loop {
            let blk = <BR as ReadBlockFromSlice>::read_from_slice(&bytes[read..], false)?;
            read += blk.size() as usize;
            blocks.push(blk);
            if read == header.blocks_len as usize {
                break;
            }
        }
        if !chk(&blocks) {
            return Ok(LookInStatus::Denied(header.size as usize));
        }
        let blocks: Vec<B> = blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>();
        let mut pkg: PacketDef<B, P, Inner> = PacketDef {
            blocks,
            payload: None,
            _pi: PhantomData,
        };
        if header.payload {
            let mut reader = std::io::BufReader::new(&bytes[read..]);
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut reader,
                        &header,
                    )? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Ok(LookInStatus::NotEnoughData(needed as usize))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Ok(LookInStatus::NotEnoughData(needed as usize))
                }
            }
        }
        Ok(LookInStatus::Accepted(header.size as usize, pkg))
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
