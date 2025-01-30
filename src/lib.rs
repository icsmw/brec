pub mod block;
pub mod error;

pub use block::*;
pub use error::*;

pub use crc32fast;

use packet::block;

#[block(df::df)]
struct MyBlock {
    field: u8,
    log_level: u8,
}

#[block(path = aaa::bbb)]
struct MyBlock2 {
    field: u8,
    log_level: u8,
}

// #[block(path = "fds", pat = 12, asf = true)]
// struct MyBlock1 {
//     field: u8,
//     log_level: u8,
// }
