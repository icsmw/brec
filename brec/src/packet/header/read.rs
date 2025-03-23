use crate::*;

impl ReadFrom for PacketHeader {
    /// Reads a `PacketHeader` from a binary stream.
    ///
    /// Validates the signature and CRC after reading. Expects fields in the following order:
    /// - Signature: `[u8; 8]`
    /// - Size: `u64` (little-endian)
    /// - Blocks length: `u64` (little-endian)
    /// - Payload flag: `u8` (0 or 1)
    /// - CRC: `u32` (little-endian)
    ///
    /// # Errors
    /// Returns:
    /// - `Error::SignatureDismatch` if the header signature is invalid.
    /// - `Error::CrcDismatch` if the CRC check fails.
    /// - I/O errors on read failure.
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != PACKET_SIG {
            return Err(Error::SignatureDismatch);
        }
        let mut size = [0u8; 8usize];
        buf.read_exact(&mut size)?;
        let size = u64::from_le_bytes(size);

        let mut blocks_len = [0u8; 8usize];
        buf.read_exact(&mut blocks_len)?;
        let blocks_len = u64::from_le_bytes(blocks_len);

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
    /// Reads a `PacketHeader` directly from a byte slice.
    ///
    /// Performs the same validation as [`ReadFrom`], but operates on a memory buffer.
    ///
    /// # Errors
    /// Returns:
    /// - `Error::NotEnoughData` if the buffer is too short.
    /// - `Error::SignatureDismatch` if the signature is invalid.
    /// - `Error::CrcDismatch` if the CRC check fails.
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

        let mut offset = 8;
        let size = u64::from_le_bytes(buf[offset..offset + 8].try_into()?);
        offset += 8;
        let blocks_len = u64::from_le_bytes(buf[offset..offset + 8].try_into()?);
        offset += 8;
        let payload = buf[offset] == 1;
        offset += 1;
        let crc = u32::from_le_bytes(buf[offset..offset + 4].try_into()?);

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
    /// Attempts to read a `PacketHeader` from a seekable stream.
    ///
    /// If there are not enough bytes available, returns a `ReadStatus::NotEnoughData` with
    /// the number of missing bytes.
    ///
    /// # Errors
    /// Propagates I/O and header validation errors.
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
    /// Attempts to read a `PacketHeader` from a buffered reader (`BufRead`).
    ///
    /// If not enough bytes are available in the internal buffer, returns a
    /// `ReadStatus::NotEnoughData`. Otherwise reads and validates the header.
    ///
    /// Consumes the number of bytes used from the buffer upon success.
    ///
    /// # Errors
    /// Returns header validation or I/O errors.
    fn try_read<T: std::io::BufRead>(reader: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let bytes = reader.fill_buf()?;
        if (bytes.len() as u64) < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PacketHeader::ssize() - bytes.len() as u64,
            ));
        }

        let header = ReadStatus::Success(PacketHeader::read(reader)?);
        reader.consume(PacketHeader::ssize() as usize);
        Ok(header)
    }
}
