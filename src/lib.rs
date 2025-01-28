pub mod error;
pub mod packet;

pub use error::*;
pub use packet::*;

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
