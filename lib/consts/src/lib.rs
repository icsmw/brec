//! Shared wire-format constants used by both `brec` and `brec_macros`.

/// Fixed size of a block signature in bytes.
pub const BLOCK_SIG_LEN: usize = 4;
/// Fixed size of the block body-length field in resilient mode.
pub const BLOCK_SIZE_FIELD_LEN: usize = 4;
/// Fixed size of block CRC in bytes.
pub const BLOCK_CRC_LEN: usize = 4;
