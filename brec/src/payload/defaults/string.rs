use crate::*;
use payload::*;

impl Size for String {
    fn size(&self) -> u64 {
        self.len() as u64
    }
}

impl Crc for String {
    fn crc(&self) -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(self.as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl Signature for String {
    fn sig() -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("String".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl ReadPayloadFrom for String {
    fn read<T: std::io::Read>(buf: &mut T, header: &PayloadHeader) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if header.sig != String::sig() {
            return Err(Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.payload_len()];
        buf.read_exact(&mut bytes)?;
        let value = String::from_utf8_lossy(&bytes).to_string();
        if header.crc != value.crc() {
            return Err(Error::CrcDismatch);
        }
        Ok(value)
    }
}

impl TryReadPayloadFrom for String {
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

impl TryReadPayloadFromBuffered for String {
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

fn write_header(src: &String, buffer: &mut [u8]) -> std::io::Result<()> {
    let blen = src.len();
    if blen > u32::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Size of payload cannot be bigger {} bytes", u32::MAX),
        ));
    }
    let blen = blen as u32;
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
    Ok(())
}

impl WriteTo for String {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write(self.as_bytes())
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write_all(self.as_bytes())
    }
}

impl WriteVectoredTo for String {
    fn slices(&self) -> std::io::Result<IoSlices> {
        let mut slices = IoSlices::default();
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        slices.add_buffered(header.to_vec());
        slices.add_slice(self.as_bytes());
        Ok(slices)
    }
}
