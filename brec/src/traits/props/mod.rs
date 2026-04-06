mod byte_block;

pub use byte_block::*;

use crate::payload::{PayloadEncoded, PayloadHooks};

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
    Self: PayloadEncoded + PayloadHooks,
{
    /// Computes CRC32 of the encoded payload.
    ///
    /// If referred encoding is available, it is used; otherwise, regular encoding is used.
    ///
    /// # Returns
    /// A 4-byte `ByteBlock` containing the CRC checksum.
    fn crc(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<ByteBlock> {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(self.encoded(ctx)?.as_slice());
        Ok(ByteBlock::Len4(hasher.finalize().to_le_bytes()))
    }

    /// Returns the size in bytes of the CRC representation produced by this trait.
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
pub trait PayloadSize: PayloadEncoded {
    /// Returns the total size (in bytes) of the payload.
    ///
    /// # Errors
    /// Returns an I/O error if size computation fails.
    fn size(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<u64> {
        Ok(self.encoded(ctx)?.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PayloadEncode, PayloadEncodeReferred, PayloadSchema};

    struct DemoPayload(Vec<u8>);

    impl PayloadSchema for DemoPayload {
        type Context<'a> = ();
    }

    impl PayloadHooks for DemoPayload {}

    impl PayloadEncode for DemoPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.0.clone())
        }
    }

    impl PayloadEncodeReferred for DemoPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(Some(self.0.as_slice()))
        }
    }

    impl PayloadCrc for DemoPayload {}
    impl PayloadSize for DemoPayload {}

    #[test]
    fn default_payload_crc_and_size_work() {
        let mut ctx = ();
        let payload = DemoPayload(vec![1, 2, 3, 4]);
        let crc = payload.crc(&mut ctx).expect("crc must work");

        assert_eq!(crc.size(), 4);
        assert_eq!(DemoPayload::crc_size(), 4);
        assert_eq!(payload.size(&mut ctx).expect("size must work"), 4);
    }
}
