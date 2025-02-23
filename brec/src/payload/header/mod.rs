mod read;
mod sreader;

use crate::*;
pub use sreader::*;

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
pub struct PayloadHeader {
    pub sig: ByteBlock,
    pub crc: ByteBlock,
    pub len: u32,
}

impl PayloadHeader {
    pub fn new<T: PayloadSignature + PayloadSize + PayloadCrc>(src: &T) -> std::io::Result<Self> {
        let len = src.size()?;
        if len > u32::MAX as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Size of payload cannot be bigger {} bytes", u32::MAX),
            ));
        }
        Ok(Self {
            sig: src.sig(),
            crc: src.crc()?,
            len: len as u32,
        })
    }
    pub fn payload_len(&self) -> usize {
        self.len as usize
    }
    pub fn size(&self) -> usize {
        1 + self.sig.size() + 1 + self.crc.size() + std::mem::size_of::<u32>()
    }
    pub fn ssize<T: PayloadSignature + PayloadSize + PayloadCrc>(
        src: &T,
    ) -> std::io::Result<usize> {
        Ok(1 + src.size()? as usize + 1 + T::crc_size() + std::mem::size_of::<u32>())
    }
    pub fn as_vec(&self) -> Vec<u8> {
        let sig = self.sig.as_slice();
        let crc = self.crc.as_slice();
        let mut buffer = Vec::new();
        // Write SIG len
        buffer.push(sig.len() as u8);
        // Write SIG
        buffer.extend_from_slice(sig);
        // Write CRC len
        buffer.push(crc.len() as u8);
        // Write CRC
        buffer.extend_from_slice(crc);
        // Write PAYLOAD len
        buffer.extend_from_slice(&self.len.to_le_bytes());
        buffer
    }
}
