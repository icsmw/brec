pub mod safemode;
pub mod unsafemode;

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
struct MyPacketPacket {
    __sig: [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
    __next: [u8; 4usize],
}
#[repr(C)]
#[derive(Debug)]
struct MyPacketPacketReferred<'a> {
    __sig: &'a [u8; 4usize],
    field: &'a u8,
    log_level: &'a u8,
    __crc: &'a u32,
    __next: &'a [u8; 4usize],
}
impl<'a> From<MyPacketPacketReferred<'a>> for MyPacket {
    fn from(packet: MyPacketPacketReferred<'a>) -> Self {
        MyPacket {
            field: *packet.field,
            log_level: *packet.log_level,
        }
    }
}
const MYPACKET: [u8; 4] = [11u8, 198u8, 4u8, 71u8];
impl<'a> crate::Packet<MyPacketPacketReferred<'a>> for MyPacketPacketReferred<'a> {
    fn sig() -> &'static [u8; 4] {
        &MYPACKET
    }
    fn read(data: &[u8]) -> Result<Option<MyPacketPacketReferred<'a>>, crate::Error> {
        use std::mem;
        if data.len() < 4 {
            return Err(crate::Error::NotEnoughtSignatureData(data.len(), 4));
        }
        if data[..4] != MYPACKET {
            return Ok(None);
        }
        if data.len() < mem::size_of::<MyPacketPacket>() {
            return Err(crate::Error::NotEnoughtData(
                data.len(),
                mem::size_of::<MyPacketPacket>(),
            ));
        }
        if data.as_ptr() as usize % std::mem::align_of::<MyPacketPacket>() != 0 {
            return Err(crate::Error::InvalidAlign(
                data.as_ptr() as usize,
                mem::size_of::<MyPacketPacket>(),
                data.as_ptr() as usize % std::mem::align_of::<MyPacketPacket>(),
            ));
        }
        let __sig = unsafe { &*(data.as_ptr() as *const [u8; 4usize]) };
        let field = unsafe { &*data.as_ptr().add(4usize) };
        let log_level = unsafe { &*data.as_ptr().add(5usize) };
        let __crc = unsafe { &*(data.as_ptr().add(6usize) as *const u32) };
        let __next = unsafe { &*(data.as_ptr().add(10usize) as *const [u8; 4usize]) };
        Ok(Some(MyPacketPacketReferred {
            __sig,
            field,
            log_level,
            __crc,
            __next,
        }))
    }
}
