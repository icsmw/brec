use crate::*;

impl ReadFrom for PacketHeader {
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

        let mut crc = [0u8; 4usize];
        buf.read_exact(&mut crc)?;
        let crc = u32::from_le_bytes(crc);
        let pkg = PacketHeader {
            blocks_len,
            size,
            payload,
            crc,
        };
        if pkg.crc.to_le_bytes() == pkg.crc() {
            Ok(pkg)
        } else {
            Err(Error::CrcDismatch)
        }
    }
}

impl ReadBlockFromSlice for PacketHeader {
    fn read_from_slice<'a>(buf: &'a [u8], _skip_sig: bool) -> Result<Self, Error>
    where
        Self: 'a + Sized,
    {
        if buf.len() < PacketHeader::ssize() as usize {
            return Err(Error::NotEnoughData(PacketHeader::ssize() as usize));
        }
        if !buf.starts_with(&PACKET_SIG) {
            return Err(Error::SignatureDismatch);
        }
        let size = u16::from_le_bytes(buf[8..10].try_into()?) as u64;
        let blocks_len = u16::from_le_bytes(buf[10..12].try_into()?) as u64;
        let payload = buf[12] == 1;
        let crc = u32::from_le_bytes(buf[13..17].try_into()?);
        let pkg = PacketHeader {
            blocks_len,
            size,
            payload,
            crc,
        };
        if pkg.crc.to_le_bytes() == pkg.crc() {
            Ok(pkg)
        } else {
            Err(Error::CrcDismatch)
        }
    }
}

impl TryReadFrom for PacketHeader {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - len));
        }
        Ok(ReadStatus::Success(PacketHeader::read(buf)?))
    }
}

impl TryReadFromBuffered for PacketHeader {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;

        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if (bytes.len() as u64) < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PacketHeader::ssize() - bytes.len() as u64,
            ));
        }
        let header = ReadStatus::Success(PacketHeader::read(&mut reader)?);
        reader.consume(PacketHeader::ssize() as usize);
        Ok(header)
    }
}
