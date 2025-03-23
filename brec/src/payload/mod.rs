mod defaults;
mod header;

pub use header::*;

/// Optional lifecycle hooks for payload encoding and decoding.
///
/// These hooks can be used to prepare the payload before serialization
/// or to perform post-processing after deserialization.
///
/// They are **never required** to do anything — by default, they are no-ops.
///
/// Implement this trait when you want to:
/// - Reset or update internal state before encoding
/// - Validate or transform data after decoding
pub trait PayloadHooks {
    /// Called before encoding begins.
    ///
    /// Can be used to perform cleanup, compute checksums, or update fields.
    fn before_encode(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    /// Called after decoding is complete.
    ///
    /// Can be used to validate, fix up or normalize decoded data.
    fn after_decode(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Trait for serializing a payload into a byte buffer.
///
/// Requires `PayloadHooks`, so `before_encode()` will always be invoked before encoding.
pub trait PayloadEncode: PayloadHooks {
    /// Encodes the payload and returns a `Vec<u8>` containing serialized bytes.
    ///
    /// # Returns
    /// The encoded byte buffer.
    ///
    /// # Errors
    /// Any I/O or serialization error encountered during encoding.
    fn encode(&self) -> std::io::Result<Vec<u8>>;
}

/// Provides an optional reference to an already-encoded payload.
///
/// This is a performance optimization: if the payload was already serialized,
/// this trait can return a reference to the existing bytes and skip re-encoding.
///
/// Useful in zero-copy or deferred encoding scenarios.
pub trait PayloadEncodeReferred {
    /// Optionally returns a reference to a pre-encoded payload.
    ///
    /// # Returns
    /// - `Some(&[u8])` if the encoded buffer is available.
    /// - `None` if the payload must be encoded with [`PayloadEncode`].
    fn encode(&self) -> std::io::Result<Option<&[u8]>>;
}

/// Trait for decoding a payload from a byte buffer.
///
/// Requires `PayloadHooks`, so `after_decode()` will always be called after decoding.
pub trait PayloadDecode<T>: PayloadHooks {
    /// Deserializes a payload from the provided byte slice.
    ///
    /// # Arguments
    /// * `buf` – The raw buffer containing the payload data.
    ///
    /// # Returns
    /// The decoded payload object.
    ///
    /// # Errors
    /// Any error encountered while decoding or validating the payload.
    fn decode(buf: &[u8]) -> std::io::Result<T>;
}
