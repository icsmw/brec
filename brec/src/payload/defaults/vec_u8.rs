use crate::{crc32fast, payload::*};

impl Size for Vec<u8> {
    fn size(&self) -> usize {
        self.len()
    }
}

impl Crc for Vec<u8> {
    fn crc(&self) -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(self);
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl Signature for Vec<u8> {
    fn sig(&self) -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("Vec<u8>".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl Payload for Vec<u8> {
    fn as_bytes(&self) -> Vec<u8> {
        self.clone()
    }
    fn from_bytes(buf: &[u8]) -> Self {
        buf.to_vec()
    }
}
