mod byte_block;

pub use byte_block::*;

use crate::payload::{PayloadEncode, PayloadEncodeReferred, PayloadHooks};

/// Trait for types that provide a static 4-byte signature.
///
/// Used for identifying or tagging structures in a binary format.
pub trait SignatureU32 {
    /// Returns a reference to a static 4-byte signature.
    fn sig() -> &'static [u8; 4];
}

/// Trait for types that can produce a CRC32 checksum.
///
/// This typically includes part of or all of the internal state of the struct.
pub trait CrcU32 {
    /// Computes a CRC32 checksum over the relevant data and returns it as 4 bytes (little-endian).
    fn crc(&self) -> [u8; 4];
}

/// Trait for payload types that support CRC calculation.
///
/// This trait requires that the type implements:
/// - `PayloadEncode`: the main payload encoding logic
/// - `PayloadEncodeReferred`: possibly optimized or referred encoding
/// - `PayloadHooks`: any pre/post encode hooks
///
/// The CRC is always returned as a `ByteBlock::Len4`.
pub trait PayloadCrc
where
    Self: PayloadEncode + PayloadHooks + PayloadEncodeReferred,
{
    /// Computes CRC32 of the encoded payload.
    ///
    /// If referred encoding is available, it is used; otherwise, regular encoding is used.
    ///
    /// # Returns
    /// A 4-byte `ByteBlock` containing the CRC checksum.
    fn crc(&self) -> std::io::Result<ByteBlock> {
        let mut hasher = crc32fast::Hasher::new();
        if let Some(bytes) = PayloadEncodeReferred::encode(self)? {
            hasher.update(bytes);
        } else {
            hasher.update(&PayloadEncode::encode(self)?);
        }
        Ok(ByteBlock::Len4(hasher.finalize().to_le_bytes()))
    }
    fn crc_size() -> usize {
        4
    }
}

/// Trait for types that can return a payload signature dynamically.
///
/// Signature is returned as a `ByteBlock`.
pub trait PayloadSignature {
    /// Returns the dynamic payload signature as a byte block.
    fn sig(&self) -> ByteBlock;
}

/// Trait for types that define a static payload signature.
///
/// Signature is returned as a `ByteBlock` and is constant for the type.
pub trait StaticPayloadSignature {
    /// Returns the static signature as a byte block.
    fn ssig() -> ByteBlock;
}

/// Trait for types with a known, constant serialized size.
pub trait StaticSize {
    /// Returns the fixed size (in bytes) of the type.
    fn ssize() -> u64;
}

/// Trait for types that can report their serialized size at runtime.
pub trait Size {
    /// Returns the size (in bytes) of the instance.
    fn size(&self) -> u64;
}

/// Trait for payload types that return size as a `Result`.
///
/// This accounts for I/O or encoding-related failures during size calculation.
pub trait PayloadSize {
    /// Returns the total size (in bytes) of the payload.
    ///
    /// # Errors
    /// Returns an I/O error if size computation fails.
    fn size(&self) -> std::io::Result<u64>;
}
