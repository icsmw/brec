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
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error> {
        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != PACKET_SIG {
            return Err(Error::SignatureDismatch(Unrecognized::payload(
                sig.to_vec(),
            )));
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
            return Err(Error::SignatureDismatch(Unrecognized::payload(
                buf[..PACKET_SIG.len()].to_vec(),
            )));
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
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error> {
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
    fn try_read<T: std::io::BufRead>(reader: &mut T) -> Result<ReadStatus<Self>, Error> {
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

#[cfg(test)]
mod tests {
    use crate::{
        CrcU32, Error, PacketHeader, ReadBlockFromSlice, ReadFrom, ReadStatus, TryReadFrom,
        TryReadFromBuffered, WriteTo,
    };
    use std::io::{BufReader, Cursor, Seek};

    fn sample_header() -> PacketHeader {
        let mut header = PacketHeader {
            size: 123,
            blocks_len: 77,
            payload: true,
            crc: 0,
        };
        header.crc = u32::from_le_bytes(header.crc());
        header
    }

    fn encoded_header() -> Vec<u8> {
        let header = sample_header();
        let mut out = Vec::new();
        header.write_all(&mut out).expect("header must serialize");
        out
    }

    #[test]
    fn read_and_read_from_slice_decode_valid_header() {
        let bytes = encoded_header();

        let mut cursor = Cursor::new(bytes.clone());
        let read = PacketHeader::read(&mut cursor).expect("read from io must work");
        assert_eq!(read.size, 123);
        assert_eq!(read.blocks_len, 77);
        assert!(read.payload);

        let from_slice =
            PacketHeader::read_from_slice(&bytes, false).expect("read from slice must work");
        assert_eq!(from_slice.size, 123);
        assert_eq!(from_slice.blocks_len, 77);
        assert!(from_slice.payload);
    }

    #[test]
    fn read_detects_signature_and_crc_errors() {
        let mut bad_sig = encoded_header();
        bad_sig[0] ^= 0xFF;
        let mut cursor = Cursor::new(bad_sig);
        assert!(matches!(
            PacketHeader::read(&mut cursor),
            Err(Error::SignatureDismatch(_))
        ));

        let mut bad_crc = encoded_header();
        let last = bad_crc.len() - 1;
        bad_crc[last] ^= 0xFF;
        let mut cursor = Cursor::new(bad_crc);
        assert!(matches!(
            PacketHeader::read(&mut cursor),
            Err(Error::CrcDismatch)
        ));
    }

    #[test]
    fn try_read_reports_not_enough_and_success_without_moving_on_not_enough() {
        let full = encoded_header();
        let short = full[..8].to_vec();
        let mut cursor = Cursor::new(short);

        match <PacketHeader as TryReadFrom>::try_read(&mut cursor)
            .expect("try_read must not fail on short input")
        {
            ReadStatus::NotEnoughData(need) => assert!(need > 0),
            ReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }
        assert_eq!(
            cursor.stream_position().expect("stream position"),
            0,
            "cursor position must remain unchanged on NotEnoughData"
        );

        let mut cursor = Cursor::new(full);
        match <PacketHeader as TryReadFrom>::try_read(&mut cursor)
            .expect("try_read must decode full header")
        {
            ReadStatus::Success(header) => {
                assert_eq!(header.size, 123);
                assert_eq!(header.blocks_len, 77);
                assert!(header.payload);
            }
            ReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
        assert_eq!(
            cursor.stream_position().expect("stream position"),
            PacketHeader::SIZE
        );
    }

    #[test]
    fn try_read_buffered_reports_not_enough_and_success() {
        let full = encoded_header();
        let short = full[..5].to_vec();

        let mut reader = BufReader::new(Cursor::new(short));
        match <PacketHeader as TryReadFromBuffered>::try_read(&mut reader)
            .expect("buffered short read must not fail")
        {
            ReadStatus::NotEnoughData(need) => assert!(need > 0),
            ReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }

        let mut reader = BufReader::new(Cursor::new(full));
        match <PacketHeader as TryReadFromBuffered>::try_read(&mut reader)
            .expect("buffered full read must succeed")
        {
            ReadStatus::Success(header) => {
                assert_eq!(header.size, 123);
                assert_eq!(header.blocks_len, 77);
                assert!(header.payload);
            }
            ReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
    }
}
