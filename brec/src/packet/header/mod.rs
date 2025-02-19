mod read;
mod write;

use crate::*;

pub const PACKET_SIG: [u8; 8] = [236u8, 37u8, 94u8, 136u8, 236u8, 37u8, 94u8, 136u8];

pub struct PacketHeader {
    /// Size of full packet (PacketHeader + Blocks + Payload)
    pub size: u64,
    /// Lenght of data, includes only Blocks
    pub blocks_len: u64,
    /// Is payload included
    pub payload: bool,
}

impl PacketHeader {
    pub const SIZE: u64 =
        (PACKET_SIG.len() + std::mem::size_of::<u16>() + std::mem::size_of::<u16>() + 1) as u64;

    pub fn get_pos(buffer: &[u8]) -> Option<usize> {
        let mut offset = 0;
        while buffer.len() > offset + PACKET_SIG.len() {
            if buffer[offset..].starts_with(&PACKET_SIG) {
                return Some(offset);
            } else {
                offset += 1;
                continue;
            }
        }
        None
    }

    pub fn is_not_enought(buffer: &[u8]) -> Option<usize> {
        if buffer.len() >= Self::SIZE as usize {
            None
        } else {
            Some(Self::SIZE as usize - buffer.len())
        }
    }
}

impl StaticSize for PacketHeader {
    fn ssize() -> u64 {
        PacketHeader::SIZE
    }
}
