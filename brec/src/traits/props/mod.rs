mod byte_block;

pub use byte_block::*;

pub trait SignatureU32 {
    fn sig() -> &'static [u8; 4];
}

pub trait CrcU32 {
    fn crc(&self) -> [u8; 4];
}

pub trait Crc {
    fn crc(&self) -> ByteBlock;
}

pub trait Signature {
    fn sig() -> ByteBlock;
}

pub trait StaticSize {
    fn ssize() -> u64;
}

pub trait Size {
    fn size(&self) -> u64;
}
