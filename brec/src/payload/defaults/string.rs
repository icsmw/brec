use crate::*;
use payload::*;

impl PayloadSize for String {
    fn size(&self) -> std::io::Result<u64> {
        Ok(self.len() as u64)
    }
}

impl PayloadCrc for String {}

impl Signature for String {
    fn sig() -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("String".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl PayloadEncode for String {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}

impl PayloadEncodeReferred for String {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Ok(Some(self.as_bytes()))
    }
}

impl PayloadDecode<String> for String {
    fn decode(buf: &[u8]) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(buf).to_string())
    }
}

impl ReadPayloadFrom<String> for String {}

impl TryReadPayloadFrom<String> for String {}

impl TryReadPayloadFromBuffered<String> for String {}

impl WritePayloadTo for String {}

impl WriteVectoredPayloadTo for String {}
