#![deny(unused_crate_dependencies)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "crypt")]
pub mod crypt;
#[cfg(feature = "napi")]
#[path = "napi/mod.rs"]
pub mod napi_feature;
#[cfg(feature = "wasm")]
#[path = "wasm/mod.rs"]
pub mod wasm_feature;
#[cfg(feature = "bincode")]
pub use bincode;
#[cfg(test)]
pub mod tests;
#[cfg(test)]
use tempfile as _;

extern crate brec_macros;

/// Maximum number of blocks allowed in a single packet.
pub const MAX_BLOCKS_COUNT: u8 = u8::MAX;

/// Shared error types used across the crate.
pub mod error;
/// Packet types, readers, rules, and related helpers.
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
pub use payload::{
    DefaultPayloadContext, PayloadDecode, PayloadEncode, PayloadEncodeReferred, PayloadHeader,
    PayloadHooks, PayloadSchema, default_payload_context,
};
pub use storage::*;

pub use crate::error::*;
#[cfg(feature = "napi")]
pub use crate::napi_feature::*;
pub use crate::packet::*;
pub use crate::traits::*;
#[cfg(feature = "wasm")]
pub use crate::wasm_feature::*;
