use crate::*;
use payload::*;

impl Size for Vec<u8> {
    fn size(&self) -> u64 {
        self.len() as u64
    }
}

impl Crc for Vec<u8> {
    fn crc(&self) -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(self);
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl Signature for Vec<u8> {
    fn sig() -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("Vec<u8>".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl ReadPayloadFrom for Vec<u8> {
    fn read<T: std::io::Read>(buf: &mut T, header: &PayloadHeader) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if header.sig != Vec::<u8>::sig() {
            return Err(Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.len()];
        buf.read_exact(&mut bytes)?;
        if header.crc != bytes.crc() {
            return Err(Error::CrcDismatch);
        }
        Ok(bytes)
    }
}

impl TryReadPayloadFrom for Vec<u8> {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        ReadPayloadFrom::read(buf, header).map(ReadStatus::Success)
    }
}

impl TryReadPayloadFromBuffered for Vec<u8> {
    fn try_read<T: std::io::Read>(
        buf: &mut T,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        ReadPayloadFrom::read(buf, header).map(ReadStatus::Success)
    }
}

fn write_header<T: std::io::Write>(src: &Vec<u8>, writer: &mut T) -> std::io::Result<usize> {
    let blen = src.len();
    if blen > u32::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Size of payload cannot be bigger {} bytes", u32::MAX),
        ));
    }
    let blen = blen as u32;
    let mut buffer = [0u8; 1 + 4 + 1 + 4 + 4];
    let mut offset = 0;
    // Write SIG len
    buffer[offset..offset + 1usize].copy_from_slice(&[4u8]);
    offset += 1usize;
    // Write SIG
    buffer[offset..offset + 4usize].copy_from_slice(Vec::<u8>::sig().as_slice());
    offset += 4usize;
    // Write CRC len
    buffer[offset..offset + 1usize].copy_from_slice(&[4u8]);
    offset += 1usize;
    // Write CRC
    buffer[offset..offset + 4usize].copy_from_slice(src.crc().as_slice());
    offset += 4usize;
    // Write PAYLOAD len
    buffer[offset..offset + 4usize].copy_from_slice(&blen.to_le_bytes());
    writer.write(&buffer)
}

impl WriteTo for Vec<u8> {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut header = [0u8; 14];
        write_header(self, &mut &mut header[..])?;
        let slices = &[std::io::IoSlice::new(&header), std::io::IoSlice::new(self)];
        writer.write_vectored(slices)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        self.write(writer)?;
        writer.write_all(&[])
    }
}
