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
    pub const LEN: usize = 1 + 4 + 1 + 4 + 4;

    pub fn payload_len(&self) -> usize {
        self.len as usize
    }
    pub fn write<T: Signature + PayloadSize + PayloadCrc>(
        src: &T,
        buffer: &mut [u8],
    ) -> std::io::Result<()> {
        let blen = src.size()?;
        if blen > u32::MAX as u64 {
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
        buffer[offset..offset + 4usize].copy_from_slice(T::sig().as_slice());
        offset += 4usize;
        // Write CRC len
        buffer[offset..offset + 1usize].copy_from_slice(&[4u8]);
        offset += 1usize;
        // Write CRC
        buffer[offset..offset + 4usize].copy_from_slice(src.crc()?.as_slice());
        offset += 4usize;
        // Write PAYLOAD len
        buffer[offset..offset + 4usize].copy_from_slice(&blen.to_le_bytes());
        Ok(())
    }
}
