mod byte_block;

pub use byte_block::*;

pub trait Crc {
    fn crc(&self) -> ByteBlock;
}

pub trait Signature {
    fn sig() -> ByteBlock;
}
