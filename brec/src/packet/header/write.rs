use crate::*;

/// Serializes a `PacketHeader` into the provided mutable buffer.
///
/// This function writes the following fields in order:
/// - Signature: `[u8; 8]`
/// - Size: `u64` (little-endian)
/// - Blocks length: `u64` (little-endian)
/// - Payload flag: `u8` (`1` if payload exists, `0` otherwise)
/// - CRC: `u32` (little-endian)
///
/// # Arguments
/// * `header` – Reference to the `PacketHeader` to serialize.
/// * `buffer` – A mutable byte slice where the header will be written.
///              Must be at least `PacketHeader::SIZE` bytes long.
fn fill_buf(header: &PacketHeader, buffer: &mut [u8]) {
    let mut offset = 0;
    buffer[offset..offset + 8usize].copy_from_slice(&PACKET_SIG);
    offset += 8usize;
    buffer[offset..offset + 8usize].copy_from_slice(&header.size.to_le_bytes());
    offset += 8usize;
    buffer[offset..offset + 8usize].copy_from_slice(&header.blocks_len.to_le_bytes());
    offset += 8usize;
    buffer[offset..offset + 1usize].copy_from_slice(&[header.payload.into()]);
    offset += 1;
    buffer[offset..offset + 4usize].copy_from_slice(&header.crc.to_le_bytes());
}

impl WriteTo for PacketHeader {
    /// Serializes and writes the `PacketHeader` into the given output stream.
    ///
    /// # Arguments
    /// * `buf` – A writer implementing `std::io::Write`.
    ///
    /// # Returns
    /// The number of bytes written (always `PacketHeader::SIZE`) on success.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if writing fails.
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; PacketHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write(&buffer)
    }

    /// Writes the entire serialized `PacketHeader`, ensuring the full write completes.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if writing fails.
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; PacketHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write_all(&buffer)
    }
}
