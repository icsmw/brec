use brec::block;
use rand::{
    distr::{Distribution, StandardUniform},
    rngs::ThreadRng,
    Rng,
};
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
impl brec::block::Signature for CustomBlockReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &CUSTOMBLOCK
    }
}
impl brec::block::Crc for CustomBlock {
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
impl brec::block::Crc for CustomBlockReferred<'_> {
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
impl brec::StaticSize for CustomBlock {
    fn static_size() -> u64 {
        283u64
    }
}
impl brec::block::Read for CustomBlock {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::block::*;
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
impl<'a> brec::block::ReadFromSlice<'a> for CustomBlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::block::*;
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != CUSTOMBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            CustomBlock::static_size() - 4
        } else {
            CustomBlock::static_size()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughData(buf.len(), required));
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
impl brec::block::TryRead for CustomBlock {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::block::*;
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(brec::ReadStatus::NotEnoughDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != CUSTOMBLOCK {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(brec::ReadStatus::DismatchSignature);
        }
        if len < CustomBlock::static_size() {
            return Ok(brec::ReadStatus::NotEnoughData(
                CustomBlock::static_size() - len,
            ));
        }
        Ok(brec::ReadStatus::Success(CustomBlock::read(buf, true)?))
    }
}
impl brec::block::TryReadBuffered for CustomBlock {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::block::*;
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(brec::ReadStatus::NotEnoughDataToReadSig(
                (4 - bytes.len()) as u64,
            ));
        }
        if !bytes.starts_with(&CUSTOMBLOCK) {
            return Ok(brec::ReadStatus::DismatchSignature);
        }
        if (bytes.len() as u64) < CustomBlock::static_size() {
            return Ok(brec::ReadStatus::NotEnoughData(
                CustomBlock::static_size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = CustomBlock::read(&mut reader, true);
        reader.consume(CustomBlock::static_size() as usize - 4);
        Ok(brec::ReadStatus::Success(blk?))
    }
}
impl brec::block::Write for CustomBlock {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        use brec::block::*;
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
        use brec::block::*;
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
impl brec::block::WriteOwned for CustomBlock {
    fn write<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<usize> {
        use brec::block::*;
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
        use brec::block::*;
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
fn main() {}
