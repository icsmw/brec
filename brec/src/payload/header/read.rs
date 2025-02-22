use crate::payload::{NextChunk, PayloadHeader, SafeHeaderReader};
use crate::*;

impl ReadFrom for PayloadHeader {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut sig_len = [0u8; 1];
        buf.read_exact(&mut sig_len)?;
        let sig_len = sig_len[0];
        ByteBlock::is_valid_capacity(sig_len)?;
        let mut sig = vec![0u8; sig_len as usize];
        buf.read_exact(&mut sig)?;
        let mut crc_len = [0u8; 1];
        buf.read_exact(&mut crc_len)?;
        let crc_len = crc_len[0];
        ByteBlock::is_valid_capacity(crc_len)?;
        let mut crc = vec![0u8; crc_len as usize];
        buf.read_exact(&mut crc)?;
        let mut len = [0u8; 4];
        buf.read_exact(&mut len)?;
        Ok(Self {
            crc: crc.try_into()?,
            len: u32::from_le_bytes(len),
            sig: sig.try_into()?,
        })
    }
}

impl TryReadFrom for PayloadHeader {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut reader = SafeHeaderReader::new(buf)?;
        let sig_len = match reader.next_u8()? {
            NextChunk::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            NextChunk::U8(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        ByteBlock::is_valid_capacity(sig_len)?;
        let sig = match reader.next_bytes(sig_len as u64)? {
            NextChunk::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            NextChunk::Bytes(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        let crc_len = match reader.next_u8()? {
            NextChunk::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            NextChunk::U8(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        ByteBlock::is_valid_capacity(crc_len)?;
        let crc = match reader.next_bytes(crc_len as u64)? {
            NextChunk::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            NextChunk::Bytes(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        let len = match reader.next_u32()? {
            NextChunk::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            NextChunk::U32(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        Ok(ReadStatus::Success(Self {
            crc: crc.try_into()?,
            len,
            sig: sig.try_into()?,
        }))
    }
}

impl TryReadFromBuffered for PayloadHeader {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::{BufRead, BufReader};
        fn ensure_available(buffer: &[u8], required: usize) -> Option<ReadStatus<PayloadHeader>> {
            if buffer.len() < required {
                Some(ReadStatus::NotEnoughData((required - buffer.len()) as u64))
            } else {
                None
            }
        }
        let mut reader = BufReader::new(buf);
        let buffer = reader.fill_buf()?;
        let mut required = 1; // Signature size (u8)
        if let Some(rs) = ensure_available(buffer, required) {
            return Ok(rs);
        }
        let sig_len = buffer[required - 1];
        ByteBlock::is_valid_capacity(sig_len)?;

        required += sig_len as usize; // Add signature len
        if let Some(rs) = ensure_available(buffer, required) {
            return Ok(rs);
        }

        required += 1; // CRC size (u8)
        if let Some(rs) = ensure_available(buffer, required) {
            return Ok(rs);
        }

        let crc_len = buffer[required - 1];
        ByteBlock::is_valid_capacity(crc_len)?;

        required += crc_len as usize; // Add CRC len
        if let Some(rs) = ensure_available(buffer, required) {
            return Ok(rs);
        }

        required += 4; // Payload length (u32)
        if let Some(rs) = ensure_available(buffer, required) {
            return Ok(rs);
        }

        let header = PayloadHeader::read(&mut reader)?;
        Ok(ReadStatus::Success(header))
    }
}
