use brec::*;
use rand::{
    distr::{Distribution, StandardUniform},
    rngs::ThreadRng,
    Rng,
};
use std::{
    fmt::{format, Debug},
    io::{BufReader, Cursor, Seek},
    ops::Deref,
};
mod extended {}
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

impl TryFrom<u8> for Level {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Level::Err),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Debug),
            3 => Ok(Level::Info),
            invalid => Err(String::new()),
        }
    }
}
impl From<&Level> for u8 {
    fn from(value: &Level) -> Self {
        match value {
            Level::Err => 0,
            Level::Warn => 1,
            Level::Debug => 2,
            Level::Info => 3,
        }
    }
}
#[repr(C)]
pub struct WithEnum {
    pub level: Level,
    data: [u8; 250],
}
#[repr(C)]
struct WithEnumReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    pub level: Level,
    data: &'a [u8; 250usize],
    __crc: &'a [u8; 4usize],
}

impl<'a> From<WithEnumReferred<'a>> for WithEnum {
    fn from(block: WithEnumReferred<'a>) -> Self {
        WithEnum {
            level: block.level,
            data: *block.data,
        }
    }
}
const WITHENUM: [u8; 4] = [97u8, 121u8, 149u8, 171u8];
impl Signature for WithEnumReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &WITHENUM
    }
}
impl brec::Crc for WithEnum {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[(&self.level).into()]);
        hasher.update(&self.data);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Crc for WithEnumReferred<'_> {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[(&self.level).into()]);
        hasher.update(self.data);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for WithEnum {
    fn size() -> u64 {
        259u64
    }
}
impl brec::Read for WithEnum {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != WITHENUM {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut level = [0u8; 1];
        buf.read_exact(&mut level)?;
        let level = Level::try_from(level[0])
            .map_err(|err| brec::Error::FailedConverting("Level".to_owned(), err))?;
        let mut data = [0u8; 250usize];
        buf.read_exact(&mut data)?;
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = WithEnum { level, data };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadFromSlice<'a> for WithEnumReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != WITHENUM {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            WithEnum::size() - 4
        } else {
            WithEnum::size()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughtData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &WITHENUM
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let level = Level::try_from(u8::from_le_bytes(buf[4usize..5usize].try_into()?))
            .map_err(|err| brec::Error::FailedConverting("Level".to_owned(), err))?;
        let data = <&[u8; 250usize]>::try_from(&buf[5usize..255usize])?;
        let __crc = <&[u8; 4usize]>::try_from(&buf[255usize..255usize + 4usize])?;
        let crc = __crc;
        let block = WithEnumReferred {
            __sig,
            level,
            data,
            __crc,
        };
        if block.crc() != *crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl brec::TryRead for WithEnum {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != WITHENUM {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::DismatchSignature);
        }
        if len < WithEnum::size() {
            return Ok(ReadStatus::NotEnoughtData(WithEnum::size() - len));
        }
        Ok(ReadStatus::Success(WithEnum::read(buf, true)?))
    }
}
impl brec::TryReadBuffered for WithEnum {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(
                (4 - bytes.len()) as u64,
            ));
        }
        if !bytes.starts_with(&WITHENUM) {
            return Ok(ReadStatus::DismatchSignature);
        }
        if (bytes.len() as u64) < WithEnum::size() {
            return Ok(ReadStatus::NotEnoughtData(
                WithEnum::size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = WithEnum::read(&mut reader, true);
        reader.consume(WithEnum::size() as usize - 4);
        Ok(ReadStatus::Success(blk?))
    }
}
impl brec::Write for WithEnum {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        buffer[offset..offset + 250usize].copy_from_slice(&self.data);
        offset += 250usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        buffer[offset..offset + 250usize].copy_from_slice(&self.data);
        offset += 250usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write_all(&buffer)
    }
}
impl brec::WriteOwned for WithEnum {
    fn write<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 250usize);
        }
        offset += 250usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 250usize);
        }
        offset += 250usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}
#[repr(C)]
pub struct CustomBlock {
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    blob_a: [u8; 100],
    blob_b: [u8; 100],
}
#[repr(C)]
struct CustomBlockReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    blob_a: &'a [u8; 100usize],
    blob_b: &'a [u8; 100usize],
    __crc: &'a [u8; 4usize],
}

impl<'a> From<CustomBlockReferred<'a>> for CustomBlock {
    fn from(block: CustomBlockReferred<'a>) -> Self {
        CustomBlock {
            field_u8: block.field_u8,
            field_u16: block.field_u16,
            field_u32: block.field_u32,
            field_u64: block.field_u64,
            field_u128: block.field_u128,
            field_i8: block.field_i8,
            field_i16: block.field_i16,
            field_i32: block.field_i32,
            field_i64: block.field_i64,
            field_i128: block.field_i128,
            field_f32: block.field_f32,
            field_f64: block.field_f64,
            field_bool: block.field_bool,
            blob_a: *block.blob_a,
            blob_b: *block.blob_b,
        }
    }
}
const CUSTOMBLOCK: [u8; 4] = [236u8, 37u8, 94u8, 136u8];
impl Signature for CustomBlockReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &CUSTOMBLOCK
    }
}
impl brec::Crc for CustomBlock {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field_u8]);
        hasher.update(&self.field_u16.to_le_bytes());
        hasher.update(&self.field_u32.to_le_bytes());
        hasher.update(&self.field_u64.to_le_bytes());
        hasher.update(&self.field_u128.to_le_bytes());
        hasher.update(&self.field_i8.to_le_bytes());
        hasher.update(&self.field_i16.to_le_bytes());
        hasher.update(&self.field_i32.to_le_bytes());
        hasher.update(&self.field_i64.to_le_bytes());
        hasher.update(&self.field_i128.to_le_bytes());
        hasher.update(&self.field_f32.to_le_bytes());
        hasher.update(&self.field_f64.to_le_bytes());
        hasher.update(&[self.field_bool as u8]);
        hasher.update(&self.blob_a);
        hasher.update(&self.blob_b);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Crc for CustomBlockReferred<'_> {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field_u8]);
        hasher.update(&self.field_u16.to_le_bytes());
        hasher.update(&self.field_u32.to_le_bytes());
        hasher.update(&self.field_u64.to_le_bytes());
        hasher.update(&self.field_u128.to_le_bytes());
        hasher.update(&self.field_i8.to_le_bytes());
        hasher.update(&self.field_i16.to_le_bytes());
        hasher.update(&self.field_i32.to_le_bytes());
        hasher.update(&self.field_i64.to_le_bytes());
        hasher.update(&self.field_i128.to_le_bytes());
        hasher.update(&self.field_f32.to_le_bytes());
        hasher.update(&self.field_f64.to_le_bytes());
        hasher.update(&[self.field_bool as u8]);
        hasher.update(self.blob_a);
        hasher.update(self.blob_b);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for CustomBlock {
    fn size() -> u64 {
        283u64
    }
}
impl brec::Read for CustomBlock {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != CUSTOMBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut field_u8 = [0u8; 1];
        buf.read_exact(&mut field_u8)?;
        let field_u8 = field_u8[0];
        let mut field_u16 = [0u8; 2usize];
        buf.read_exact(&mut field_u16)?;
        let field_u16 = u16::from_le_bytes(field_u16);
        let mut field_u32 = [0u8; 4usize];
        buf.read_exact(&mut field_u32)?;
        let field_u32 = u32::from_le_bytes(field_u32);
        let mut field_u64 = [0u8; 8usize];
        buf.read_exact(&mut field_u64)?;
        let field_u64 = u64::from_le_bytes(field_u64);
        let mut field_u128 = [0u8; 16usize];
        buf.read_exact(&mut field_u128)?;
        let field_u128 = u128::from_le_bytes(field_u128);
        let mut field_i8 = [0u8; 1usize];
        buf.read_exact(&mut field_i8)?;
        let field_i8 = i8::from_le_bytes(field_i8);
        let mut field_i16 = [0u8; 2usize];
        buf.read_exact(&mut field_i16)?;
        let field_i16 = i16::from_le_bytes(field_i16);
        let mut field_i32 = [0u8; 4usize];
        buf.read_exact(&mut field_i32)?;
        let field_i32 = i32::from_le_bytes(field_i32);
        let mut field_i64 = [0u8; 8usize];
        buf.read_exact(&mut field_i64)?;
        let field_i64 = i64::from_le_bytes(field_i64);
        let mut field_i128 = [0u8; 16usize];
        buf.read_exact(&mut field_i128)?;
        let field_i128 = i128::from_le_bytes(field_i128);
        let mut field_f32 = [0u8; 4usize];
        buf.read_exact(&mut field_f32)?;
        let field_f32 = f32::from_le_bytes(field_f32);
        let mut field_f64 = [0u8; 8usize];
        buf.read_exact(&mut field_f64)?;
        let field_f64 = f64::from_le_bytes(field_f64);
        let mut field_bool = [0u8; 1usize];
        buf.read_exact(&mut field_bool)?;
        let field_bool = field_bool[0] != 0;
        let mut blob_a = [0u8; 100usize];
        buf.read_exact(&mut blob_a)?;
        let mut blob_b = [0u8; 100usize];
        buf.read_exact(&mut blob_b)?;
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = CustomBlock {
            field_u8,
            field_u16,
            field_u32,
            field_u64,
            field_u128,
            field_i8,
            field_i16,
            field_i32,
            field_i64,
            field_i128,
            field_f32,
            field_f64,
            field_bool,
            blob_a,
            blob_b,
        };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadFromSlice<'a> for CustomBlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != CUSTOMBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            CustomBlock::size() - 4
        } else {
            CustomBlock::size()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughtData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &CUSTOMBLOCK
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let field_u8 = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let field_u16 = u16::from_le_bytes(buf[5usize..7usize].try_into()?);
        let field_u32 = u32::from_le_bytes(buf[7usize..11usize].try_into()?);
        let field_u64 = u64::from_le_bytes(buf[11usize..19usize].try_into()?);
        let field_u128 = u128::from_le_bytes(buf[19usize..35usize].try_into()?);
        let field_i8 = i8::from_le_bytes(buf[35usize..36usize].try_into()?);
        let field_i16 = i16::from_le_bytes(buf[36usize..38usize].try_into()?);
        let field_i32 = i32::from_le_bytes(buf[38usize..42usize].try_into()?);
        let field_i64 = i64::from_le_bytes(buf[42usize..50usize].try_into()?);
        let field_i128 = i128::from_le_bytes(buf[50usize..66usize].try_into()?);
        let field_f32 = f32::from_le_bytes(buf[66usize..70usize].try_into()?);
        let field_f64 = f64::from_le_bytes(buf[70usize..78usize].try_into()?);
        let field_bool = u8::from_le_bytes(buf[78usize..79usize].try_into()?) == 1;
        let blob_a = <&[u8; 100usize]>::try_from(&buf[79usize..179usize])?;
        let blob_b = <&[u8; 100usize]>::try_from(&buf[179usize..279usize])?;
        let __crc = <&[u8; 4usize]>::try_from(&buf[279usize..279usize + 4usize])?;
        let crc = __crc;
        let block = CustomBlockReferred {
            __sig,
            field_u8,
            field_u16,
            field_u32,
            field_u64,
            field_u128,
            field_i8,
            field_i16,
            field_i32,
            field_i64,
            field_i128,
            field_f32,
            field_f64,
            field_bool,
            blob_a,
            blob_b,
            __crc,
        };
        if block.crc() != *crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl brec::TryRead for CustomBlock {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != CUSTOMBLOCK {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::DismatchSignature);
        }
        if len < CustomBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(CustomBlock::size() - len));
        }
        Ok(ReadStatus::Success(CustomBlock::read(buf, true)?))
    }
}
impl brec::TryReadBuffered for CustomBlock {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(
                (4 - bytes.len()) as u64,
            ));
        }
        if !bytes.starts_with(&CUSTOMBLOCK) {
            return Ok(ReadStatus::DismatchSignature);
        }
        if (bytes.len() as u64) < CustomBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(
                CustomBlock::size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = CustomBlock::read(&mut reader, true);
        reader.consume(CustomBlock::size() as usize - 4);
        Ok(ReadStatus::Success(blk?))
    }
}
impl brec::Write for CustomBlock {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 283usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&CUSTOMBLOCK);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_u8]);
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_u16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_u32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_u64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_u128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 1usize].copy_from_slice(&self.field_i8.to_le_bytes());
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_i16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_i32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_i64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_i128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_f32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_f64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_bool as u8]);
        offset += 1usize;
        buffer[offset..offset + 100usize].copy_from_slice(&self.blob_a);
        offset += 100usize;
        buffer[offset..offset + 100usize].copy_from_slice(&self.blob_b);
        offset += 100usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 283usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&CUSTOMBLOCK);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_u8]);
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_u16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_u32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_u64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_u128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 1usize].copy_from_slice(&self.field_i8.to_le_bytes());
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_i16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_i32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_i64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_i128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_f32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_f64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_bool as u8]);
        offset += 1usize;
        buffer[offset..offset + 100usize].copy_from_slice(&self.blob_a);
        offset += 100usize;
        buffer[offset..offset + 100usize].copy_from_slice(&self.blob_b);
        offset += 100usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write_all(&buffer)
    }
}
impl brec::WriteOwned for CustomBlock {
    fn write<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 283usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&CUSTOMBLOCK);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_u8]);
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_u16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_u32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_u64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_u128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 1usize].copy_from_slice(&self.field_i8.to_le_bytes());
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_i16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_i32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_i64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_i128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_f32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_f64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_bool as u8]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.blob_a.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.blob_b.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 283usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&CUSTOMBLOCK);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_u8]);
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_u16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_u32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_u64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_u128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 1usize].copy_from_slice(&self.field_i8.to_le_bytes());
        offset += 1usize;
        buffer[offset..offset + 2usize].copy_from_slice(&self.field_i16.to_le_bytes());
        offset += 2usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_i32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_i64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 16usize].copy_from_slice(&self.field_i128.to_le_bytes());
        offset += 16usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.field_f32.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.field_f64.to_le_bytes());
        offset += 8usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[self.field_bool as u8]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.blob_a.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.blob_b.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}

pub enum Block {
    WithEnum(WithEnum),
    CustomBlock(CustomBlock),
}
impl brec::TryRead for Block {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let result = <WithEnum as brec::TryReadBuffered>::try_read(buf)?;
        if !match result {
            brec::ReadStatus::DismatchSignature => true,
            _ => false,
        } {
            return Ok(result.map(Block::WithEnum));
        }
        let result = <CustomBlock as brec::TryReadBuffered>::try_read(buf)?;
        if !match result {
            brec::ReadStatus::DismatchSignature => true,
            _ => false,
        } {
            return Ok(result.map(Block::CustomBlock));
        }
        Ok(brec::ReadStatus::DismatchSignature)
    }
}
impl CustomBlock {
    pub fn rand() -> Self {
        let mut rng = rand::rng();
        fn slice<T>(rng: &ThreadRng) -> [T; 100]
        where
            StandardUniform: Distribution<T>,
            T: Debug,
        {
            rng.clone()
                .random_iter()
                .take(100)
                .collect::<Vec<T>>()
                .try_into()
                .expect("Expected 100 elements")
        }
        Self {
            field_u8: rng.random(),
            field_u16: rng.random(),
            field_u32: rng.random(),
            field_u64: rng.random(),
            field_u128: rng.random(),
            field_i8: rng.random(),
            field_i16: rng.random(),
            field_i32: rng.random(),
            field_i64: rng.random(),
            field_i128: rng.random(),
            field_f32: rng.random(),
            field_f64: rng.random(),
            field_bool: rng.random_bool(1.0 / 3.0),
            blob_a: slice::<u8>(&rng),
            blob_b: slice::<u8>(&rng),
        }
    }
}
