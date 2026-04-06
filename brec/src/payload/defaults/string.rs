use crate::*;
use payload::*;

/// `String` is supported as a default payload type in `brec`.
///
/// It provides complete support for encoding, decoding, signature verification,
/// CRC validation, and vectored writing. The string is treated as UTF-8 encoded data
/// and is serialized as-is without any additional framing or length prefix.
impl PayloadSize for String {
    fn size(&self, _: &mut Self::Context<'_>) -> std::io::Result<u64> {
        Ok(self.len() as u64)
    }
}

impl PayloadCrc for String {}

impl PayloadSignature for String {
    fn sig(&self) -> ByteBlock {
        <String as StaticPayloadSignature>::ssig()
    }
}

impl StaticPayloadSignature for String {
    fn ssig() -> ByteBlock {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update("String".as_bytes());
        ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl PayloadSchema for String {
    type Context<'a> = DefaultPayloadContext;
}

impl PayloadEncode for String {
    fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}

impl PayloadEncodeReferred for String {
    fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
        Ok(Some(self.as_bytes()))
    }
}

impl PayloadDecode<String> for String {
    fn decode(buf: &[u8], _: &mut Self::Context<'_>) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(buf).to_string())
    }
}

impl ReadPayloadFrom<String> for String {}

impl TryReadPayloadFrom<String> for String {}

impl TryReadPayloadFromBuffered<String> for String {}

impl WritePayloadWithHeaderTo for String {}

impl WriteVectoredPayloadWithHeaderTo for String {}

impl PayloadHooks for String {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn string_payload_roundtrip_and_signature() {
        let mut ctx = default_payload_context();
        let mut payload = String::from("hello");

        let header = PayloadHeader::new(&payload, &mut ctx).expect("header must build");
        assert_eq!(header.payload_len(), 5);
        assert_eq!(payload.sig(), <String as StaticPayloadSignature>::ssig());

        let mut encoded = Vec::new();
        payload
            .write_all(&mut encoded, &mut ctx)
            .expect("write_all must work");
        assert!(encoded.len() > header.payload_len());

        let mut cursor = Cursor::new(encoded);
        let parsed = <PayloadHeader as ReadFrom>::read(&mut cursor).expect("header must parse");
        let restored =
            <String as ReadPayloadFrom<String>>::read(&mut cursor, &parsed, &mut ctx)
                .expect("payload must parse");
        assert_eq!(restored, "hello");
    }
}
