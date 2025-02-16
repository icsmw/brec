use crate::*;

impl ReadFrom for PackageHeader {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != PACKET_SIG {
            return Err(Error::SignatureDismatch);
        }
        let mut size = [0u8; 2usize];
        buf.read_exact(&mut size)?;
        let size = u16::from_le_bytes(size) as u64;

        let mut blocks_len = [0u8; 2usize];
        buf.read_exact(&mut blocks_len)?;
        let blocks_len = u16::from_le_bytes(blocks_len) as u64;

        let mut payload = [0u8; 1usize];
        buf.read_exact(&mut payload)?;
        let payload = payload[0] == 1;

        Ok(PackageHeader {
            blocks_len,
            size,
            payload,
        })
    }
}

impl<'a> ReadBlockFromSlice<'a> for PackageHeader {
    fn read_from_slice(buf: &'a [u8], _skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if buf.len() < PackageHeader::ssize() as usize {
            return Err(Error::NotEnoughData(
                buf.len(),
                PackageHeader::ssize() as usize,
            ));
        }
        if !buf.starts_with(&PACKET_SIG) {
            return Err(Error::SignatureDismatch);
        }
        let size = u16::from_le_bytes(buf[8..10].try_into()?) as u64;
        let blocks_len = u16::from_le_bytes(buf[10..12].try_into()?) as u64;
        let payload = buf[12] == 1;
        Ok(PackageHeader {
            blocks_len,
            size,
            payload,
        })
    }
}

impl TryReadFrom for PackageHeader {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < PackageHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PackageHeader::ssize() - len));
        }
        Ok(ReadStatus::Success(PackageHeader::read(buf)?))
    }
}

impl TryReadFromBuffered for PackageHeader {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;

        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if (bytes.len() as u64) < PackageHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PackageHeader::ssize() - bytes.len() as u64,
            ));
        }
        let header = ReadStatus::Success(PackageHeader::read(&mut reader)?);
        reader.consume(PackageHeader::ssize() as usize);
        Ok(header)
    }
}
