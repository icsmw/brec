mod read;
mod sreader;

use crate::*;
pub use sreader::*;

/// Represents the header of a payload, containing metadata for verification and identification.
///
/// This header precedes the actual payload bytes and provides:
/// - a **unique signature** identifying the payload format/type
/// - a **CRC checksum** for integrity validation
/// - the **length of the payload**
///
/// # Binary Layout
/// The payload header is stored in the following sequence:
///
/// | Field            | Size (bytes)     | Description                         |
/// |------------------|------------------|-------------------------------------|
/// | Signature length | `1`              | One of: `4, 8, 16, 32, 64, 128`     |
/// | Signature        | variable         | Payload type identifier             |
/// | CRC length       | `1`              | One of: `4, 8, 16, 32, 64, 128`     |
/// | CRC              | variable         | Checksum for the payload            |
/// | Payload length   | `4` (`u32`)      | Payload size in bytes (LE)          |
///
/// This flexible structure supports various signature/CRC sizes while keeping the layout
/// parseable and consistent.
pub struct PayloadHeader {
    /// Unique signature identifying the payload format.
    pub sig: ByteBlock,

    /// CRC checksum over the payload content.
    pub crc: ByteBlock,

    /// Length of the payload (in bytes).
    pub len: u32,
}

impl PayloadHeader {
    /// Constructs a new `PayloadHeader` using the given payload object.
    ///
    /// This function extracts the signature, CRC, and size from the payload
    /// by calling the corresponding traits.
    ///
    /// # Errors
    /// Returns an I/O error if the payload size exceeds `u32::MAX`.
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

    /// Returns the payload length as a `usize`, for buffer allocation.
    pub fn payload_len(&self) -> usize {
        self.len as usize
    }

    /// Computes the full size of this header in bytes.
    ///
    /// This includes:
    /// - 1 byte for signature length
    /// - signature bytes
    /// - 1 byte for CRC length
    /// - CRC bytes
    /// - 4 bytes for payload length
    pub fn size(&self) -> usize {
        1 + self.sig.size() + 1 + self.crc.size() + std::mem::size_of::<u32>()
    }

    /// Computes the size of a `PayloadHeader` that would be generated from a given object.
    ///
    /// This is a static utility that avoids constructing the header instance.
    ///
    /// # Returns
    /// Total byte length of the resulting header.
    pub fn ssize<T: PayloadSignature + PayloadSize + PayloadCrc>(
        src: &T,
    ) -> std::io::Result<usize> {
        Ok(1 + src.sig().size() + 1 + T::crc_size() + std::mem::size_of::<u32>())
    }

    /// Serializes the header into a binary vector.
    ///
    /// The output format matches the layout described above and is suitable
    /// for writing before the payload body.
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
