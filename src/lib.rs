pub mod error;
pub mod packet;

pub use error::*;
pub use packet::*;

pub use crc32fast;

struct MyPacket {
    field: u8,
    log_level: u8,
}
#[repr(C)]
#[derive(Debug)]
struct MyPacketReferred<'a> {
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
    __next: &'a [u8; 4usize],
}
impl<'a> From<MyPacketReferred<'a>> for MyPacket {
    fn from(packet: MyPacketReferred<'a>) -> Self {
        MyPacket {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYPACKET: [u8; 4] = [11u8, 198u8, 4u8, 71u8];

impl crate::Crc for MyPacket {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = crate::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl crate::Size for MyPacket {
    fn size(&self) -> usize {
        14usize
    }
}
impl<'a> crate::Packet<'a, MyPacketReferred<'a>> for MyPacketReferred<'a> {
    fn sig() -> &'static [u8; 4] {
        &MYPACKET
    }
    fn read(data: &'a [u8]) -> Result<Option<MyPacketReferred<'a>>, crate::Error> {
        use std::mem;
        if data.len() < 4 {
            return Err(crate::Error::NotEnoughtSignatureData(data.len(), 4));
        }
        if data[..4] != MYPACKET {
            return Ok(None);
        }
        if data.len() < mem::size_of::<MyPacket>() {
            return Err(crate::Error::NotEnoughtData(
                data.len(),
                mem::size_of::<MyPacket>(),
            ));
        }
        let __sig = <&[u8; 4usize]>::try_from(&data[0usize..4usize])?;
        let field = u8::from_le_bytes(data[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(data[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(data[6usize..10usize].try_into()?);
        let __next = <&[u8; 4usize]>::try_from(&data[10usize..14usize])?;
        Ok(Some(MyPacketReferred {
            __sig,
            field,
            log_level,
            __crc,
            __next,
        }))
    }
}
impl crate::Write for MyPacket {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(buf.write(&MYPACKET)?
            + buf.write(&[self.field])?
            + buf.write(&[self.log_level])?
            + buf.write(&self.crc())?)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYPACKET)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
