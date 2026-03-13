mod defaults;
mod header;

pub use header::*;

/// Default payload context used by payloads that require no runtime state.
pub type DefaultPayloadContext = ();

/// Returns the default payload context value, equivalent to `()`.
pub const fn default_payload_context() -> DefaultPayloadContext {}

/// Associates a payload type with the runtime context it expects during processing.
pub trait PayloadSchema {
    /// Runtime context passed into payload encode, decode, and size operations.
    type Context<'a>;
}

/// Represents an encoded payload body, either borrowed from the original value
/// or materialized into an owned buffer.
pub enum EncodedPayload<'a> {
    /// Bytes borrowed from an existing encoded representation.
    Borrowed(&'a [u8]),
    /// Bytes materialized into an owned buffer.
    Owned(Vec<u8>),
}

impl EncodedPayload<'_> {
    /// Returns the encoded bytes as a contiguous slice.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes.as_slice(),
        }
    }

    /// Returns the length of the encoded payload body.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Returns `true` when the encoded payload body is empty.
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
}

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
pub trait PayloadEncode: PayloadHooks + PayloadSchema {
    /// Serializes the payload body into a standalone byte buffer.
    fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>>;
}

/// Provides an optional reference to an already-encoded payload.
///
/// This is a performance optimization: if the payload was already serialized,
/// this trait can return a reference to the existing bytes and skip re-encoding.
///
/// Useful in zero-copy or deferred encoding scenarios.
pub trait PayloadEncodeReferred: PayloadSchema {
    /// Returns a borrowed encoded payload body when one is already available.
    fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>>;
}

/// Resolves the payload body into either a borrowed slice or an owned buffer.
///
/// This provides a single internal representation that can be reused for
/// computing the payload length, CRC, and actual write operations without
/// repeating the same encode step multiple times.
pub trait PayloadEncoded: PayloadEncode + PayloadEncodeReferred {
    /// Returns the encoded payload body.
    fn encoded(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<EncodedPayload<'_>> {
        if let Some(bytes) = PayloadEncodeReferred::encode(self, ctx)? {
            Ok(EncodedPayload::Borrowed(bytes))
        } else {
            Ok(EncodedPayload::Owned(PayloadEncode::encode(self, ctx)?))
        }
    }
}

impl<T> PayloadEncoded for T where T: PayloadEncode + PayloadEncodeReferred {}

/// Trait for decoding a payload from a byte buffer.
///
/// Requires `PayloadHooks`, so `after_decode()` will always be called after decoding.
pub trait PayloadDecode<T>: PayloadHooks + PayloadSchema {
    /// Reconstructs a payload value from encoded payload bytes.
    fn decode(buf: &[u8], ctx: &mut Self::Context<'_>) -> std::io::Result<T>;
}
