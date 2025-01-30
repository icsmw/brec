pub mod block;
pub mod error;

pub use block::*;
pub use error::*;

pub use crc32fast;

struct MyBlock {
    field: u8,
    log_level: u8,
}
#[repr(C)]
#[derive(Debug)]
struct MyBlockReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
}
impl<'a> From<MyBlockReferred<'a>> for MyBlock {
    fn from(packet: MyBlockReferred<'a>) -> Self {
        MyBlock {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYBLOCK: [u8; 4] = [254u8, 32u8, 165u8, 251u8];
impl MyBlockReferred<'_> {
    pub fn sig() -> &'static [u8; 4] {
        &MYBLOCK
    }
}
impl crate::Crc for MyBlock {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = crate::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl crate::Size for MyBlock {
    fn size(&self) -> usize {
        10usize
    }
}
impl crate::Read for MyBlock {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 4];
        buf.read_exact(&mut sig)?;
        if sig != MYBLOCK {
            return Err(crate::Error::SignatureDismatch);
        }
        let mut field = [0u8; 1];
        buf.read_exact(&mut field)?;
        let field = field[0];
        let mut log_level = [0u8; 1];
        buf.read_exact(&mut log_level)?;
        let log_level = log_level[0];
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let packet = MyBlock { field, log_level };
        if packet.crc() != crc {
            return Err(crate::Error::CrcDismatch);
        }
        Ok(packet)
    }
}
impl<'a> crate::ReadFromSlice<'a> for MyBlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8]) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        if buf.len() < 4 {
            return Err(crate::Error::NotEnoughtSignatureData(buf.len(), 4));
        }
        if buf[..4] != MYBLOCK {
            return Err(crate::Error::SignatureDismatch);
        }
        if buf.len() < std::mem::size_of::<MyBlock>() {
            return Err(crate::Error::NotEnoughtData(
                buf.len(),
                std::mem::size_of::<MyBlock>(),
            ));
        }
        let __sig = <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?;
        let field = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(buf[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(buf[6usize..10usize].try_into()?);
        Ok(MyBlockReferred {
            __sig,
            field,
            log_level,
            __crc,
        })
    }
}
impl crate::Write for MyBlock {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(buf.write(&MYBLOCK)?
            + buf.write(&[self.field])?
            + buf.write(&[self.log_level])?
            + buf.write(&self.crc())?)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYBLOCK)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
