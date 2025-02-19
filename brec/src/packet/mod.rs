mod header;

pub use header::*;

use crate::*;
use std::{io::BufRead, marker::PhantomData};

pub trait BlockReferredDef<'a, B: BlockDef>:
    ReadBlockFromSlice<'a> + Size + Sized + Into<B>
{
}

pub trait BlockDef:
    ReadBlockFrom + ReadFrom + TryReadFrom + TryReadFromBuffered + WriteTo + WriteVectoredTo + Size
{
}

pub trait PayloadInnerDef: Sized + ExtractPayloadFrom<Self> {}

pub trait PayloadDef<Inner: PayloadInnerDef>:
    ExtractPayloadFrom<Inner>
    + TryExtractPayloadFrom<Inner>
    + TryExtractPayloadFromBuffered<Inner>
    + WritingPayloadTo
    + WritingVectoredPayloadTo
    + PayloadSize
{
}

pub struct PacketReferred<
    'a,
    B: BlockDef,
    BR: BlockReferredDef<'a, B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub blocks: Vec<BR>,
    pub header: PacketHeader,
    pub len: u64,
    _b: PhantomData<&'a B>,
    _p: PhantomData<P>,
    _i: PhantomData<Inner>,
}

pub struct PacketBufReader<
    'a,
    R: std::io::Read,
    B: BlockDef,
    BR: BlockReferredDef<'a, B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    inner: std::io::BufReader<R>,
    referred: Option<PacketReferred<'a, B, BR, P, Inner>>,
}

impl<
        'a,
        R: std::io::Read,
        B: BlockDef,
        BR: BlockReferredDef<'a, B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > PacketBufReader<'a, R, B, BR, P, Inner>
{
    pub fn new(inner: R) -> Self {
        Self {
            inner: std::io::BufReader::new(inner),
            referred: None,
        }
    }
    pub fn buffer(&self) -> &[u8] {
        if self.referred.is_some() {
            &[]
        } else {
            self.inner.buffer()
        }
    }
    pub fn capacity(&self) -> usize {
        if self.referred.is_some() {
            0
        } else {
            self.inner.capacity()
        }
    }
    pub fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.referred.is_some() {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Waiting for accept or refuse",
            ))
        } else {
            self.inner.fill_buf()
        }
    }
    pub fn consume(&mut self, amt: usize) {
        if self.referred.is_none() {
            self.inner.consume(amt)
        }
    }
    pub fn look_in(
        &mut self,
    ) -> Result<ReadStatus<&'a PacketReferred<'_, B, BR, P, Inner>>, Error> {
        let inner = self.inner.fill_buf()?;
        let len = inner.len() as u64;
        if len < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - len));
        }
        let header = PacketHeader::read_from_slice(inner, false)?;
        if header.size > len {
            return Ok(ReadStatus::NotEnoughData(header.size - len));
        }
        let mut blocks = Vec::new();
        let mut read = PacketHeader::ssize() as usize;
        loop {
            let blk = <BR as ReadBlockFromSlice>::read_from_slice(&inner[read..], false)?;
            read += blk.size() as usize;
            blocks.push(blk);
            if read == header.blocks_len as usize {
                break;
            }
        }
        self.referred = Some(PacketReferred {
            blocks,
            header,
            len,
            _i: PhantomData,
            _b: PhantomData,
            _p: PhantomData,
        });
        Ok(ReadStatus::Success(self.referred.as_ref().unwrap()))
    }
    pub fn accept(&mut self) -> Result<Packet<B, P, Inner>, Error> {
        let Some(referred) = self.referred.take() else {
            return Err(Error::NoPendingPacket);
        };
        let blocks = referred
            .blocks
            .into_iter()
            .map(|blk| blk.into())
            .collect::<Vec<B>>();
        let mut pkg = Packet::new(blocks, None);
        if referred.header.payload {
            // We are not using inner BufReader to prevent usage inner methods and consuming
            // bytes on inner buffer
            let bytes = self.inner.fill_buf()?;
            let mut cursor = std::io::Cursor::new(&bytes[referred.header.blocks_len as usize..]);
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut cursor)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut cursor,
                        &header,
                    )? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Err(Error::NotEnoughData(
                                referred.len as usize,
                                needed as usize,
                            ))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(referred.len as usize, needed as usize))
                }
            }
        }
        self.inner.consume(referred.header.size as usize);
        Ok(pkg)
    }
    pub fn refuse(&mut self) -> Result<(), std::io::Error> {
        let Some(referred) = self.referred.take() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "No waiting for accept or refuse packet",
            ));
        };
        self.consume(referred.header.size as usize);
        Ok(())
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
//             .consume((blks.header.size - PacketHeader::ssize()) as usize);
//         pkg.blocks = blks.blocks;
//         Ok(pkg)
//     }
// }

pub enum LookInStatus<T> {
    Accepted(usize, T),
    Denied(usize),
    NotEnoughData(usize),
}

pub struct Packet<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    pub blocks: Vec<B>,
    pub payload: Option<Inner>,
    _pi: PhantomData<P>,
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> Packet<B, P, Inner> {
    pub fn new(blocks: Vec<B>, payload: Option<Inner>) -> Self {
        Self {
            blocks,
            payload,
            _pi: PhantomData,
        }
    }
    pub fn look_in<'a, BR, F>(
        bytes: &'a [u8],
        chk: F,
    ) -> Result<LookInStatus<Packet<B, P, Inner>>, Error>
    where
        BR: BlockReferredDef<'a, B>,
        F: FnOnce(&[BR]) -> bool,
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
        let blocks = blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>();
        let mut pkg: Packet<B, P, Inner> = Packet {
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
        let header = PacketHeader::read(buf)?;
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
            let payload = <P as ExtractPayloadFrom<Inner>>::read(buf, &header)?;
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
        let header = match <PacketHeader as TryReadFrom>::try_read(buf)? {
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
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(buf, &header) {
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
        if available < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - available));
        }
        let header = PacketHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        reader.consume(PacketHeader::ssize() as usize);
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
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut reader,
                        &header,
                    )? {
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
        reader.consume((header.size - PacketHeader::ssize()) as usize);
        Ok(ReadStatus::Success(pkg))
    }
}
