use crate::*;

fn fill_buf(header: &PackageHeader, buffer: &mut [u8]) {
    let mut offset = 0;
    buffer[offset..offset + 8usize].copy_from_slice(&PACKET_SIG);
    offset += 8usize;
    buffer[offset..offset + 2usize].copy_from_slice(&header.size.to_le_bytes());
    offset += 2usize;
    buffer[offset..offset + 2usize].copy_from_slice(&header.blocks_len.to_le_bytes());
    offset += 2usize;
    buffer[offset..offset + 1usize].copy_from_slice(&[header.payload.into()]);
}
impl WriteTo for PackageHeader {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; PackageHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; PackageHeader::SIZE as usize];
        fill_buf(self, &mut buffer);
        buf.write_all(&buffer)
    }
}
