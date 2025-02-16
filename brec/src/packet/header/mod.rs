mod read;
mod write;

use crate::*;

pub const PACKET_SIG: [u8; 8] = [236u8, 37u8, 94u8, 136u8, 236u8, 37u8, 94u8, 136u8];

pub struct PackageHeader {
    pub size: u64,
    pub blocks_len: u64,
    pub payload: bool,
}

impl PackageHeader {
    const SIZE: u64 =
        (PACKET_SIG.len() + std::mem::size_of::<u16>() + std::mem::size_of::<u16>() + 1) as u64;
}

impl StaticSize for PackageHeader {
    fn ssize() -> u64 {
        PackageHeader::SIZE
    }
}
