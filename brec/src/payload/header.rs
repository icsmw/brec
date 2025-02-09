use crate::payload::*;
use crate::*;

/// Represents the header of a payload, containing metadata for verification and identification.
///
/// # Structure
///
/// The `Header` consists of the following sequentially stored fields:
///
/// - **[1 byte]** - Length of the signature (`4, 8, 16, 32, 64, 128`)
/// - **[4 to 128 bytes]** - Signature (`ByteBlock`)
/// - **[1 byte]** - Length of the CRC (`4, 8, 16, 32, 64, 128`)
/// - **[4 to 128 bytes]** - CRC (`ByteBlock`)
/// - **[4 bytes]** - Payload length (`u32`)
///
/// This structure allows for flexible signature and CRC sizes while maintaining a fixed layout
/// for efficient parsing and validation.
///
/// The `Header` is essential for ensuring the integrity of the payload by providing a unique signature
/// and a CRC for error detection.
///
/// # Fields
///
/// - `sig` - Unique signature identifying the payload format.
/// - `crc` - CRC checksum of the payload for integrity verification.
/// - `len` - Length of the payload in bytes.
pub struct Header {
    pub sig: ByteBlock,
    pub crc: ByteBlock,
    pub len: u32,
}

impl Header {
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

struct SafeReader<'a, T: std::io::Read + std::io::Seek> {
    buf: &'a mut T,
    spos: u64,
    read: u64,
    len: u64,
}

impl<'a, T: std::io::Read + std::io::Seek> SafeReader<'a, T> {
    pub fn new(buf: &'a mut T) -> Result<Self, Error> {
        let spos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - spos;
        buf.seek(std::io::SeekFrom::Start(spos))?;
        Ok(Self {
            spos,
            len,
            buf,
            read: 0,
        })
    }
    pub fn next_u8(&mut self) -> Result<Next, Error> {
        if self.len < self.read + 1 {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(Next::NotEnoughData(self.read + 1 - self.len));
        }
        let mut dest = [0u8; 1];
        self.buf.read_exact(&mut dest)?;
        self.read += 1;
        Ok(Next::U8(dest[0]))
    }
    pub fn next_u32(&mut self) -> Result<Next, Error> {
        if self.len < self.read + 4u64 {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(Next::NotEnoughData(self.read + 4u64 - self.len));
        }
        let mut dest = [0u8; 4];
        self.buf.read_exact(&mut dest)?;
        self.read += 4u64;
        Ok(Next::U32(u32::from_le_bytes(dest)))
    }
    pub fn next_bytes(&mut self, capacity: u64) -> Result<Next, Error> {
        if self.len < self.read + capacity {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(Next::NotEnoughData(self.read + capacity - self.len));
        }
        let mut dest = vec![0u8; capacity as usize];
        self.buf.read_exact(&mut dest)?;
        self.read += capacity;
        Ok(Next::Bytes(dest))
    }
}

enum Next {
    NotEnoughData(u64),
    U8(u8),
    U32(u32),
    Bytes(Vec<u8>),
}

impl TryRead for Header {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut reader = SafeReader::new(buf)?;
        let sig_len = match reader.next_u8()? {
            Next::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            Next::U8(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        ByteBlock::is_valid_capacity(sig_len)?;
        let sig = match reader.next_bytes(sig_len as u64)? {
            Next::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            Next::Bytes(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        let crc_len = match reader.next_u8()? {
            Next::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            Next::U8(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        ByteBlock::is_valid_capacity(crc_len)?;
        let crc = match reader.next_bytes(crc_len as u64)? {
            Next::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            Next::Bytes(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        let len = match reader.next_u32()? {
            Next::NotEnoughData(n) => return Ok(ReadStatus::NotEnoughData(n)),
            Next::U32(v) => v,
            _ => return Err(Error::FailToReadPayloadHeader),
        };
        Ok(ReadStatus::Success(Self {
            crc: crc.try_into()?,
            len,
            sig: sig.try_into()?,
        }))
    }
}

impl TryReadBuffered for Header {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::{BufRead, BufReader};
        fn ensure_available(buffer: &[u8], required: usize) -> Option<ReadStatus<Header>> {
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

        let header = Header::read(&mut reader)?;
        Ok(ReadStatus::Success(header))
    }
}
