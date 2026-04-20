#![deny(unused_crate_dependencies)]
#![doc = include_str!("../README.md")]

#[cfg(test)]
pub mod tests;
#[cfg(test)]
use tempfile as _;

#[cfg(feature = "crypt")]
pub mod crypt;
#[cfg(feature = "bincode")]
pub use bincode;

extern crate brec_macros;

/// Maximum number of blocks allowed in a single packet.
pub const MAX_BLOCKS_COUNT: u8 = u8::MAX;

/// Shared error types used across the crate.
pub mod error;
pub mod integrations;
/// Packet types, readers, rules, and related helpers (including resilient-mode parsing paths).
pub mod packet;
/// Payload traits, headers, and default payload helpers.
pub mod payload;
/// Convenient reexports for typical user code.
pub mod prelude;
/// Packet storage, readers, writers, and observer support.
pub mod storage;
/// Low-level traits used by generated and handwritten protocol types.
pub mod traits;

pub use brec_macros::*;
pub use crc32fast;
#[cfg(feature = "crypt")]
pub use crypt::{BricCryptCodec, CryptAlgorithm, CryptEnvelopeRecord};
#[cfg(feature = "csharp")]
pub use integrations::csharp_feature;
#[cfg(feature = "csharp")]
pub use integrations::csharp_feature::{
    CSharpConvert, CSharpError, CSharpFieldHint, CSharpFieldHintId, CSharpObject, CSharpObjectMap,
    CSharpValue, FromCSharpValue,
};
#[cfg(feature = "java")]
pub use integrations::java_feature;
#[cfg(feature = "java")]
pub use integrations::java_feature::*;
#[cfg(feature = "napi")]
pub use integrations::napi_feature;
#[cfg(feature = "napi")]
pub use integrations::napi_feature::*;
#[cfg(feature = "wasm")]
pub use integrations::wasm_feature;
#[cfg(feature = "wasm")]
pub use integrations::wasm_feature::*;
pub use payload::{
    DefaultPayloadContext, PayloadDecode, PayloadEncode, PayloadEncodeReferred, PayloadHeader,
    PayloadHooks, PayloadSchema, default_payload_context,
};
pub use storage::*;

pub use crate::error::*;
pub use crate::packet::*;
pub use crate::traits::*;
