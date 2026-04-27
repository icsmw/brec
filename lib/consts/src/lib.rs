//! Shared wire-format constants used across the public crate and generator crates.

/// Fixed size of a block signature in bytes.
pub const BLOCK_SIG_LEN: usize = 4;
/// Fixed size of the block body-length field in resilient mode.
pub const BLOCK_SIZE_FIELD_LEN: usize = 4;
/// Fixed size of block CRC in bytes.
pub const BLOCK_CRC_LEN: usize = 4;
/// Maximum number of blocks allowed in a single packet.
pub const MAX_BLOCKS_COUNT: u8 = u8::MAX;
pub const PAYLOAD_FIELD_NAME: &str = "payload";
pub const BLOCKS_FIELD_NAME: &str = "blocks";
