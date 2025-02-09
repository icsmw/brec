use crate::{crc32fast, payload::*};

impl Size for String {
    fn size(&self) -> usize {
        self.len()
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

impl Read for String {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if ByteBlock::Len4(sig) != String::sig() {
                return Err(Error::SignatureDismatch);
            }
        }
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let mut len = [0u8; 4];
        buf.read_exact(&mut len)?;
        let len = u32::from_le_bytes(len);
        let mut bytes = Vec::with_capacity(len);
        buf.read_exact(&mut bytes)?;
        let value = String::from_utf8_lossy(&bytes).to_string();
        if ByteBlock::Len4(crc) != value.crc() {
            return Err(Error::CrcDismatch);
        }
        Ok(value)
    }
}

impl TryRead for String {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(ReadStatus::NotEnoughDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig)?;
        if ByteBlock::Len4(sig) != String::sig() {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::DismatchSignature);
        }

        if len < WithEnum::size() {
            return Ok(ReadStatus::NotEnoughData(WithEnum::size() - len));
        }
        Ok(ReadStatus::Success(WithEnum::read(buf, true)?))
    }
}
impl brec::TryReadBuffered for WithEnum {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(ReadStatus::NotEnoughDataToReadSig((4 - bytes.len()) as u64));
        }
        if !bytes.starts_with(&WITHENUM) {
            return Ok(ReadStatus::DismatchSignature);
        }
        if (bytes.len() as u64) < WithEnum::size() {
            return Ok(ReadStatus::NotEnoughData(
                WithEnum::size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = WithEnum::read(&mut reader, true);
        reader.consume(WithEnum::size() as usize - 4);
        Ok(ReadStatus::Success(blk?))
    }
}
impl brec::Write for WithEnum {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        buffer[offset..offset + 250usize].copy_from_slice(&self.data);
        offset += 250usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        buffer[offset..offset + 250usize].copy_from_slice(&self.data);
        offset += 250usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.crc());
        writer.write_all(&buffer)
    }
}
impl brec::WriteOwned for WithEnum {
    fn write<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<usize> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 250usize);
        }
        offset += 250usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<()> {
        let mut buffer = [0u8; 259usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&WITHENUM);
        offset += 4usize;
        buffer[offset..offset + 1usize].copy_from_slice(&[(&self.level).into()]);
        offset += 1usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.data.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 250usize);
        }
        offset += 250usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}
