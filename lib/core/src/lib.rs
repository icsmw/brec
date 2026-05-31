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

/// Default maximum payload body length accepted by generated protocols.
pub const DEFAULT_MAX_PAYLOAD_LEN: u32 = 1024 * 1024;

/// Default maximum packet body length accepted by generated protocols.
pub const DEFAULT_MAX_PACKET_LEN: u64 = (DEFAULT_MAX_PAYLOAD_LEN as u64) * 2;

/// Default initial allocation used by packet buffer readers.
pub const DEFAULT_INITIAL_PACKET_BUFFER_CAPACITY: usize = 64 * 1024;

/// Shared error types used across the crate.
pub mod error;
/// Feature-gated integration helpers for C#, Node.js, WASM, and Java bridges.
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

#[cfg(feature = "csharp")]
pub use brec_csharp_lib as csharp_feat;
#[cfg(feature = "java")]
pub use brec_java_gen_macro::Java;
#[cfg(feature = "java")]
pub use brec_java_lib as java_feat;
pub use brec_macros::*;
#[cfg(feature = "napi")]
pub use brec_node_gen_macro::Napi;
#[cfg(feature = "napi")]
pub use brec_node_lib as napi_feat;
#[cfg(feature = "wasm")]
pub use brec_wasm_gen_macro::Wasm;
#[cfg(feature = "wasm")]
pub use brec_wasm_lib as wasm_feat;
pub use crc32fast;
#[cfg(feature = "crypt")]
pub use crypt::{CryptAlgorithm, CryptCodec, CryptEnvelopeRecord};
pub use payload::{
    DefaultProtocolContext, PayloadDecode, PayloadEncode, PayloadEncodeReferred, PayloadHeader,
    PayloadHooks, ProtocolSchema, default_payload_context,
};
pub use storage::*;

pub use crate::error::*;
pub use crate::packet::*;
pub use crate::traits::*;
