use crate::payload::{NextChunk, PayloadHeader, SafeHeaderReader};
use crate::*;

impl ReadFrom for PayloadHeader {
    /// Reads a complete `PayloadHeader` from the stream.
    ///
    /// This implementation assumes the entire header is available and performs no pre-checks
    /// on input length. It will fail immediately if any part of the header cannot be read.
    ///
    /// # Field Order
    /// 1. Signature length (1 byte)
    /// 2. Signature (`sig_len` bytes)
    /// 3. CRC length (1 byte)
    /// 4. CRC (`crc_len` bytes)
    /// 5. Payload length (4 bytes, LE)
    ///
    /// # Errors
    /// - `Error::InvalidCapacity` if signature or CRC length is invalid
    /// - `std::io::Error` if reading fails
    /// - Any conversion error from `ByteBlock::try_into()`
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
    /// Tries to read a `PayloadHeader` from a seekable stream using a safe, chunked approach.
    ///
    /// Uses `SafeHeaderReader` and `NextChunk` to avoid over-reading when data is incomplete.
    ///
    /// # Returns
    /// - `ReadStatus::Success(header)` if fully parsed
    /// - `ReadStatus::NotEnoughData(n)` if more bytes are required
    /// - `Error::FailToReadPayloadHeader` if structure is invalid or inconsistent
    /// - `Error::InvalidCapacity` for unsupported signature or CRC lengths
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
    /// Tries to read a `PayloadHeader` from a buffered reader (`BufRead`) without consuming it.
    ///
    /// This implementation works purely on the internal buffer (via `fill_buf`) and returns
    /// `NotEnoughData` if more bytes are required to complete the header.
    ///
    /// After confirming that enough bytes are available, it delegates to `PayloadHeader::read()`
    /// using a `Cursor`.
    ///
    /// # Returns
    /// - `ReadStatus::Success(header)` — fully parsed header
    /// - `ReadStatus::NotEnoughData(bytes)` — indicates how many more bytes are needed
    /// - `Error::InvalidCapacity` if signature or CRC size is unsupported
    fn try_read<T: std::io::BufRead>(reader: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        /// Helper function used in `try_read` to early-return if not enough bytes are in buffer.
        ///
        /// It calculates the required number of bytes and returns `ReadStatus::NotEnoughData` if needed.
        fn ensure_available(buffer: &[u8], required: usize) -> Option<ReadStatus<PayloadHeader>> {
            if buffer.len() < required {
                Some(ReadStatus::NotEnoughData((required - buffer.len()) as u64))
            } else {
                None
            }
        }
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

        let header = PayloadHeader::read(&mut std::io::Cursor::new(buffer))?;
        Ok(ReadStatus::Success(header))
    }
}
