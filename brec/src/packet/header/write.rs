use crate::*;

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
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; PacketHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; PacketHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write_all(&buffer)
    }
}
