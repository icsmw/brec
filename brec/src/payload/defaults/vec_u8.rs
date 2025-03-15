use crate::*;
use payload::*;

impl PayloadSize for Vec<u8> {
    fn size(&self) -> std::io::Result<u64> {
        Ok(self.len() as u64)
    }
}

impl PayloadCrc for Vec<u8> {}

impl PayloadSignature for Vec<u8> {
    fn sig(&self) -> ByteBlock {
        <Vec<u8> as StaticPayloadSignature>::ssig()
    }
}

impl StaticPayloadSignature for Vec<u8> {
    fn ssig() -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("Vec<u8>".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl PayloadEncode for Vec<u8> {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        Ok(self.clone())
    }
}

impl PayloadEncodeReferred for Vec<u8> {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Ok(Some(self))
    }
}

impl PayloadDecode<Vec<u8>> for Vec<u8> {
    fn decode(buf: &[u8]) -> std::io::Result<Vec<u8>> {
        Ok(buf.to_vec())
    }
}

impl ReadPayloadFrom<Vec<u8>> for Vec<u8> {}

impl TryReadPayloadFrom<Vec<u8>> for Vec<u8> {}

impl TryReadPayloadFromBuffered<Vec<u8>> for Vec<u8> {}

impl WritePayloadWithHeaderTo for Vec<u8> {}

impl WriteVectoredPayloadWithHeaderTo for Vec<u8> {}

impl PayloadHooks for Vec<u8> {}
