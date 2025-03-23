mod read;
mod write;

use crate::*;

/// Packet signature marker.
///
/// This constant is used to identify the beginning of a packet header in a binary stream.
/// It serves as a unique byte pattern that helps to recognize and validate the start of a packet.
pub const PACKET_SIG: [u8; 8] = [236u8, 37u8, 94u8, 136u8, 236u8, 37u8, 94u8, 136u8];

/// Packet header structure.
///
/// The packet header precedes the actual content of a packet in the binary stream.
/// A complete packet consists of:
/// - A sequence of blocks (from 0 to 255), stored consecutively.
/// - An optional payload, appended after the blocks.
#[derive(Debug)]
pub struct PacketHeader {
    /// Total size of the packet, excluding the `PacketHeader` itself (in bytes).
    pub size: u64,

    /// Total length (in bytes) of all the blocks contained in the packet.
    pub blocks_len: u64,

    /// Indicates whether the packet includes a payload.
    pub payload: bool,

    /// CRC checksum of the header itself (not the full packet).
    pub crc: u32,
}

impl PacketHeader {
    /// The total size of the serialized `PacketHeader` in bytes.
    ///
    /// This includes:
    /// - Signature (`PACKET_SIG`)
    /// - Total packet size (`u64`)
    /// - Blocks length (`u64`)
    /// - Payload presence flag (`u8`)
    /// - Header CRC (`u32`)
    pub const SIZE: u64 = (
        // Signature
        PACKET_SIG.len()
        // Length of packet
        + std::mem::size_of::<u64>()
        // Length of blocks
        + std::mem::size_of::<u64>()
        // Payload existing flag
        + 1
        // Crc
        + std::mem::size_of::<u32>()
    ) as u64;

    /// Constructs a new `PacketHeader` from a list of blocks and an optional payload.
    ///
    /// # Arguments
    /// * `blocks` - A slice of `BlockDef` items that will be included in the packet.
    /// * `payload` - An optional payload that follows the blocks, implementing `PayloadInnerDef`.
    ///
    /// # Returns
    /// A new `PacketHeader` with computed `size`, `blocks_len`, `payload` flag, and CRC.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if computing the payload size fails.
    pub fn new<B: BlockDef, Inner: PayloadInnerDef>(
        blocks: &[B],
        payload: Option<&Inner>,
    ) -> std::io::Result<Self> {
        let blocks_len: u64 = blocks.iter().map(|blk| blk.size()).sum();
        let payload_len: u64 = payload
            .as_ref()
            .map(|payload| Self::payload_size(*payload))
            .unwrap_or(Ok(0))?;
        let size = blocks_len + payload_len;
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&size.to_le_bytes());
        hasher.update(&blocks_len.to_le_bytes());
        hasher.update(payload.map(|_| &[1]).unwrap_or(&[0]));
        let crc = hasher.finalize();
        Ok(Self {
            size,
            blocks_len,
            payload: payload.is_some(),
            crc,
        })
    }

    /// Calculates the total size of the payload, including its header and body.
    ///
    /// # Arguments
    /// * `payload` - A reference to the payload implementing `PayloadInnerDef`.
    ///
    /// # Returns
    /// The total payload size in bytes (`header + body`), or an error if size calculation fails.
    pub fn payload_size<Inner: PayloadInnerDef>(payload: &Inner) -> std::io::Result<u64> {
        let payload_header_len: u64 = PayloadHeader::ssize(payload).map(|s| s as u64)?;
        let payload_body_len: u64 = payload.size()?;
        Ok(payload_body_len + payload_header_len)
    }

    /// Finds the offset of a packet header in the provided buffer by scanning for the signature.
    ///
    /// # Arguments
    /// * `buffer` - A byte slice in which to search for the packet signature.
    ///
    /// # Returns
    /// The offset of the first occurrence of `PACKET_SIG`, or `None` if not found.
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

    /// Checks if the buffer contains enough bytes to read a full packet header.
    ///
    /// # Arguments
    /// * `buffer` - A byte slice to be checked.
    ///
    /// # Returns
    /// `None` if the buffer is large enough to contain a complete header;
    /// otherwise returns the number of missing bytes.
    pub fn is_not_enought(buffer: &[u8]) -> Option<usize> {
        if buffer.len() >= Self::SIZE as usize {
            None
        } else {
            Some(Self::SIZE as usize - buffer.len())
        }
    }
}

impl StaticSize for PacketHeader {
    /// Returns the static size (in bytes) of the serialized `PacketHeader`.
    ///
    /// This is a constant value defined by the layout of the header structure.
    fn ssize() -> u64 {
        PacketHeader::SIZE
    }
}

impl CrcU32 for PacketHeader {
    /// Computes the CRC32 checksum of the header fields.
    ///
    /// The checksum is calculated over the following fields:
    /// - `size` (as little-endian bytes)
    /// - `blocks_len` (as little-endian bytes)
    /// - `payload` flag (`1` if present, `0` otherwise)
    ///
    /// # Returns
    /// A 4-byte CRC32 checksum in little-endian order.
    fn crc(&self) -> [u8; 4] {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.size.to_le_bytes());
        hasher.update(&self.blocks_len.to_le_bytes());
        hasher.update(if self.payload { &[1] } else { &[0] });
        hasher.finalize().to_le_bytes()
    }
}
