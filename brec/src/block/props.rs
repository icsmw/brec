pub use crate::*;

pub trait Signature {
    fn sig() -> &'static [u8; 4];
}

pub trait Crc {
    fn crc(&self) -> [u8; 4];
}

pub trait Size {
    fn size() -> u64;
}
