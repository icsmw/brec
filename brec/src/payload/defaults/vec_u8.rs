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
        hasher.update("String".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl ReadPayloadFrom for Vec<u8> {
    fn read<T: std::io::Read>(buf: &mut T, header: &PayloadHeader) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if header.sig != String::sig() {
            return Err(Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.payload_len()];
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

fn write_header(src: &Vec<u8>, buffer: &mut [u8]) -> std::io::Result<()> {
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

impl WriteTo for Vec<u8> {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write(self)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write_all(self)
    }
}

impl WriteVectoredTo for Vec<u8> {
    fn slices(&self) -> std::io::Result<IoSlices> {
        let mut slices = IoSlices::default();
        let mut header = [0u8; PayloadHeader::LEN];
        write_header(self, &mut header)?;
        slices.add_buffered(header.to_vec());
        slices.add_slice(self);
        Ok(slices)
    }
}
