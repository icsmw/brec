mod byte_block;

pub use byte_block::*;

use crate::payload::{PayloadEncode, PayloadEncodeReferred, PayloadHooks};

pub trait SignatureU32 {
    fn sig() -> &'static [u8; 4];
}

pub trait CrcU32 {
    fn crc(&self) -> [u8; 4];
}

pub trait PayloadCrc
where
    Self: PayloadEncode + PayloadHooks + PayloadEncodeReferred,
{
    fn crc(&self) -> std::io::Result<ByteBlock> {
        let mut hasher = crc32fast::Hasher::new();
        if let Some(bytes) = PayloadEncodeReferred::encode(self)? {
            hasher.update(bytes);
        } else {
            hasher.update(&PayloadEncode::encode(self)?);
        }
        Ok(ByteBlock::Len4(hasher.finalize().to_le_bytes()))
    }
    fn crc_size() -> usize {
        4
    }
}

pub trait PayloadSignature {
    fn sig(&self) -> ByteBlock;
}

pub trait StaticPayloadSignature {
    fn ssig() -> ByteBlock;
}

pub trait StaticSize {
    fn ssize() -> u64;
}

pub trait Size {
    fn size(&self) -> u64;
}

pub trait PayloadSize {
    fn size(&self) -> std::io::Result<u64>;
}
