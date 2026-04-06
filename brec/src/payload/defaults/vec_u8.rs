use crate::*;
use payload::*;

/// `Vec<u8>` is supported as a default payload type in `brec`.
///
/// This payload represents raw binary data. It supports full encoding/decoding,
/// CRC validation, signature identification, and efficient vectored writing.
/// No transformation or framing is applied - the raw byte content is stored and restored as-is.
impl PayloadSize for Vec<u8> {
    fn size(&self, _: &mut Self::Context<'_>) -> std::io::Result<u64> {
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

impl PayloadSchema for Vec<u8> {
    type Context<'a> = DefaultPayloadContext;
}

impl PayloadEncode for Vec<u8> {
    fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
        Ok(self.clone())
    }
}

impl PayloadEncodeReferred for Vec<u8> {
    fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
        Ok(Some(self))
    }
}

impl PayloadDecode<Vec<u8>> for Vec<u8> {
    fn decode(buf: &[u8], _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
        Ok(buf.to_vec())
    }
}

impl ReadPayloadFrom<Vec<u8>> for Vec<u8> {}

impl TryReadPayloadFrom<Vec<u8>> for Vec<u8> {}

impl TryReadPayloadFromBuffered<Vec<u8>> for Vec<u8> {}

impl WritePayloadWithHeaderTo for Vec<u8> {}

impl WriteVectoredPayloadWithHeaderTo for Vec<u8> {}

impl PayloadHooks for Vec<u8> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn vec_u8_payload_roundtrip_and_signature() {
        let mut ctx = default_payload_context();
        let mut payload = vec![1_u8, 2, 3, 4];

        let header = PayloadHeader::new(&payload, &mut ctx).expect("header must build");
        assert_eq!(header.payload_len(), 4);
        assert_eq!(payload.sig(), <Vec<u8> as StaticPayloadSignature>::ssig());

        let mut encoded = Vec::new();
        payload
            .write_all(&mut encoded, &mut ctx)
            .expect("write_all must work");
        assert!(encoded.len() > header.payload_len());

        let mut cursor = Cursor::new(encoded);
        let parsed = <PayloadHeader as ReadFrom>::read(&mut cursor).expect("header must parse");
        let restored = <Vec<u8> as ReadPayloadFrom<Vec<u8>>>::read(&mut cursor, &parsed, &mut ctx)
            .expect("payload must parse");
        assert_eq!(restored, vec![1, 2, 3, 4]);
    }
}
