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
    field: &'a u8,
    log_level: &'a u8,
    __crc: &'a u32,
    __next: &'a [u8; 4usize],
}
impl<'a> From<MyPacketReferred<'a>> for MyPacket {
    fn from(packet: MyPacketReferred<'a>) -> Self {
        MyPacket {
            field: *packet.field,
            log_level: *packet.log_level,
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
        if data.as_ptr() as usize % std::mem::align_of::<MyPacket>() != 0 {
            return Err(crate::Error::InvalidAlign(
                data.as_ptr() as usize,
                mem::size_of::<MyPacket>(),
                data.as_ptr() as usize % std::mem::align_of::<MyPacket>(),
            ));
        }
        let __sig = unsafe { &*(data.as_ptr() as *const [u8; 4usize]) };
        let field = unsafe { &*data.as_ptr().add(4usize) };
        let log_level = unsafe { &*data.as_ptr().add(5usize) };
        let __crc = unsafe { &*(data.as_ptr().add(6usize) as *const u32) };
        let __next = unsafe { &*(data.as_ptr().add(10usize) as *const [u8; 4usize]) };
        Ok(Some(MyPacketReferred {
            __sig,
            field,
            log_level,
            __crc,
            __next,
        }))
    }
}
