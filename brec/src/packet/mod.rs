mod header;

pub use header::*;

use crate::*;
use std::{io::BufRead, marker::PhantomData};

pub trait BlockReferredDef<'a, B: BlockDef>:
    ReadBlockFrom
    + ReadBlockFromSlice<'a>
    + ReadFrom
    + TryReadFrom
    + TryReadFromBuffered
    + WriteTo
    + WriteVectoredTo
    + Size
    + Sized
    + Into<B>
{
}

pub trait BlockDef:
    ReadBlockFrom + ReadFrom + TryReadFrom + TryReadFromBuffered + WriteTo + WriteVectoredTo + Size
{
}

pub trait PayloadInnerDef:
    Sized + PayloadDecode<Self> + Signature + PayloadCrc + ReadPayloadFrom<Self> + PayloadEncode
{
}

pub trait PayloadDef<Inner: PayloadInnerDef>:
    ReadPayloadFrom<Inner>
    + TryReadPayloadFrom<Inner>
    + TryReadPayloadFromBuffered<Inner>
    + WritePayloadTo
    + WriteVectoredPayloadTo
    + Signature
    + PayloadCrc
    + PayloadSize
    + PayloadDecode<Self>
{
}

pub struct PacketReferred<
    'a,
    B: BlockDef,
    BR,
    T: std::io::Read,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> where
    BR: for<'b> BlockReferredDef<'b, B>,
{
    pub blocks: Vec<BR>,
    pub reader: std::io::BufReader<&'a mut T>,
    pub header: PackageHeader,
    pub len: u64,
    _b: PhantomData<B>,
    _p: PhantomData<P>,
    _i: PhantomData<Inner>,
}

impl<'a, B: BlockDef, BR, T: std::io::Read, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
    PacketReferred<'a, B, BR, T, P, Inner>
where
    BR: for<'b> BlockReferredDef<'b, B>,
{
    pub fn read(
        buffer: &'a mut T,
    ) -> Result<ReadStatus<Option<PacketReferred<'a, B, BR, T, P, Inner>>>, Error>
    where
        Self: Sized,
    {
        let mut reader = std::io::BufReader::new(buffer);
        let bytes = reader.fill_buf()?;
        let len = bytes.len() as u64;
        if len < PackageHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PackageHeader::ssize() - len));
        }
        let header = PackageHeader::read_from_slice(bytes, false)?;
        if header.size > len {
            return Ok(ReadStatus::NotEnoughData(header.size - len));
        }
        let mut blocks = Vec::new();
        let mut read = PackageHeader::ssize() as usize;
        loop {
            let blk = <BR as ReadBlockFromSlice>::read_from_slice(&bytes[read..], false)?;
            read += blk.size() as usize;
            blocks.push(blk);
            if read == header.blocks_len as usize {
                break;
            }
        }
        Ok(ReadStatus::Success(Some(PacketReferred {
            blocks,
            reader,
            header,
            len,
            _b: PhantomData::default(),
            _p: PhantomData::default(),
            _i: PhantomData::default(),
        })))
    }

    pub fn refuse(&mut self) {
        self.reader.consume(self.header.size as usize);
    }

    pub fn accept(mut self) -> Result<Packet<B, P, Inner>, Error> {
        let blocks = self
            .blocks
            .into_iter()
            .map(|blk| blk.into())
            .collect::<Vec<B>>();
        let mut pkg: Packet<B, P, Inner> = Packet::default();
        pkg.blocks = blocks;
        if self.header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut self.reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryReadPayloadFromBuffered<Inner>>::try_read(
                        &mut self.reader,
                        &header,
                    )? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Err(Error::NotEnoughData(self.len as usize, needed as usize))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(self.len as usize, needed as usize))
                }
            }
        }
        self.reader.consume(self.header.size as usize);
        return Ok(pkg);
    }
}

// impl<
//         'a,
//         B: BlockDef,
//         BR: BlockReferredDef<'a>,
//         P: PayloadDef<Inner>,
//         Inner: PayloadInnerDef,
//         T: std::io::Read,
//     > TryFrom<PacketReferred<'a, BR, T>> for Packet<B, P, Inner>
// {
//     type Error = Error;

//     fn try_from(mut blks: PacketReferred<'a, BR, T>) -> Result<Self, Error> {
//         let mut pkg = Packet::default();
//         if blks.header.payload {
//             match <PayloadHeader as TryReadFromBuffered>::try_read(&mut blks.reader)? {
//                 ReadStatus::Success(header) => {
//                     match <P as TryReadPayloadFromBuffered<Inner>>::try_read(
//                         &mut blks.reader,
//                         &header,
//                     )? {
//                         ReadStatus::Success(payload) => {
//                             pkg.payload = Some(payload);
//                         }
//                         ReadStatus::NotEnoughData(needed) => {
//                             return Err(Error::NotEnoughData(0, needed as usize));
//                         }
//                     }
//                 }
//                 ReadStatus::NotEnoughData(needed) => {
//                     return Err(Error::NotEnoughData(0, needed as usize))
//                 }
//             }
//         }
//         blks.reader
//             .consume((blks.header.size - PackageHeader::ssize()) as usize);
//         pkg.blocks = blks.blocks;
//         Ok(pkg)
//     }
// }

pub struct Packet<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    pub blocks: Vec<B>,
    pub payload: Option<Inner>,
    _pi: PhantomData<P>,
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> Packet<B, P, Inner> {
    pub fn touch<'a, T: std::io::Read, BR, F: Fn(&[BR]) -> bool>(
        buffer: &'a mut T,
        chk: F,
    ) -> Result<ReadStatus<Option<Packet<B, P, Inner>>>, Error>
    where
        BR: for<'b> BlockReferredDef<'b, B>,
        Self: Sized,
    {
        let mut reader = std::io::BufReader::new(buffer);
        let bytes = reader.fill_buf()?;
        let available = bytes.len() as u64;
        if available < PackageHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PackageHeader::ssize() - available,
            ));
        }
        let header = PackageHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        let mut blocks = Vec::new();
        let mut read = PackageHeader::ssize() as usize;
        loop {
            let blk = <BR as ReadBlockFromSlice>::read_from_slice(&bytes[read..], false)?;
            read += blk.size() as usize;
            blocks.push(blk);
            if read == header.blocks_len as usize {
                break;
            }
        }
        if !chk(&blocks) {
            reader.consume(header.size as usize);
            return Ok(ReadStatus::Success(None));
        }
        let blocks = blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>();
        let mut pkg: Packet<B, P, Inner> = Packet::default();
        pkg.blocks = blocks;
        if header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryReadPayloadFromBuffered<Inner>>::try_read(&mut reader, &header)?
                    {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Ok(ReadStatus::NotEnoughData(needed))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(available as usize, needed as usize))
                }
            }
        }
        reader.consume(header.size as usize);
        return Ok(ReadStatus::Success(Some(pkg)));
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> Default for Packet<B, P, Inner> {
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            payload: None,
            _pi: PhantomData,
        }
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> ReadFrom for Packet<B, P, Inner> {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let header = PackageHeader::read(buf)?;
        let mut pkg = Packet::default();
        let mut read = 0;
        loop {
            // TODO: Error::SignatureDismatch should be covered in enum's context
            let blk = <B as ReadFrom>::read(buf)?;
            read += blk.size();
            pkg.blocks.push(blk);
            if read == header.blocks_len {
                break;
            }
        }
        if header.payload {
            let header = <PayloadHeader as ReadFrom>::read(buf)?;
            let payload = <P as ReadPayloadFrom<Inner>>::read(buf, &header)?;
            pkg.payload = Some(payload);
        }
        Ok(pkg)
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFrom
    for Packet<B, P, Inner>
{
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let available = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        let header = match <PackageHeader as TryReadFrom>::try_read(buf)? {
            ReadStatus::NotEnoughData(needed) => return Ok(ReadStatus::NotEnoughData(needed)),
            ReadStatus::Success(header) => header,
        };
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        let mut pkg = Packet::default();
        let mut read = 0;
        loop {
            // TODO: Error::SignatureDismatch should be covered in enum's context
            match <B as TryReadFromBuffered>::try_read(buf) {
                Ok(ReadStatus::Success(blk)) => {
                    read += blk.size();
                    pkg.blocks.push(blk);
                    if read == header.blocks_len {
                        break;
                    }
                }
                Ok(ReadStatus::NotEnoughData(needed)) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Ok(ReadStatus::NotEnoughData(needed));
                }
                Err(err) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(err);
                }
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(buf)? {
                ReadStatus::Success(header) => {
                    match <P as TryReadPayloadFromBuffered<Inner>>::try_read(buf, &header) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
                        Ok(ReadStatus::NotEnoughData(needed)) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Ok(ReadStatus::NotEnoughData(needed));
                        }
                        Err(err) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Err(err);
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(Error::NotEnoughData(0, needed as usize));
                }
            }
        }
        Ok(ReadStatus::Success(pkg))
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFromBuffered
    for Packet<B, P, Inner>
{
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        let available = bytes.len() as u64;
        if available < PackageHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PackageHeader::ssize() - available,
            ));
        }
        let header = PackageHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        reader.consume(PackageHeader::ssize() as usize);
        let mut pkg = Packet::default();
        let mut read = 0;
        loop {
            // TODO: Error::SignatureDismatch should be covered in enum's context
            match <B as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(blk) => {
                    read += blk.size();
                    pkg.blocks.push(blk);
                    if read == header.blocks_len {
                        break;
                    }
                }
                ReadStatus::NotEnoughData(needed) => return Ok(ReadStatus::NotEnoughData(needed)),
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut reader)? {
                ReadStatus::Success(header) => {
                    match <P as TryReadPayloadFromBuffered<Inner>>::try_read(&mut reader, &header)?
                    {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Ok(ReadStatus::NotEnoughData(needed))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(available as usize, needed as usize))
                }
            }
        }
        reader.consume((header.size - PackageHeader::ssize()) as usize);
        Ok(ReadStatus::Success(pkg))
    }
}
