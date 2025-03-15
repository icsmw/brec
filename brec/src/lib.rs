#[cfg(feature = "build")]
pub mod build;
#[cfg(feature = "bincode")]
pub use bincode;
#[cfg(feature = "build")]
pub use build::*;

extern crate packet as packet_macro;

pub const MAX_BLOCKS_COUNT: u8 = u8::MAX;

pub mod error;
pub mod packet;
pub mod payload;
pub mod prelude;
pub mod storage;
pub mod traits;

pub use crc32fast;
pub use packet_macro::*;
pub use payload::{
    PayloadDecode, PayloadEncode, PayloadEncodeReferred, PayloadHeader, PayloadHooks,
};
pub use r#include::*;
pub use storage::*;

pub use crate::error::*;
pub use crate::packet::*;
pub use crate::traits::*;
