#[repr(C)]
struct MyPayload {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}
impl brec::Signature for MyPayload {
    fn sig() -> brec::ByteBlock {
        brec::ByteBlock::Len4([178u8, 246u8, 54u8, 106u8])
    }
}
impl brec::PayloadEncode for MyPayload {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        brec::bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadEncodeReferred for MyPayload {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Ok(None)
    }
}
impl brec::PayloadDecode<MyPayload> for MyPayload {
    fn decode(buf: &[u8]) -> std::io::Result<MyPayload> {
        brec::bincode::deserialize(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadCrc for MyPayload {}
impl brec::PayloadSize for MyPayload {
    fn size(&self) -> std::io::Result<u64> {
        brec::bincode::serialized_size(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::ReadPayloadFrom<MyPayload> for MyPayload {}
impl brec::TryReadPayloadFrom<MyPayload> for MyPayload {}
impl brec::TryReadPayloadFromBuffered<MyPayload> for MyPayload {}
impl brec::WritePayloadTo for MyPayload {}
impl brec::WriteVectoredPayloadTo for MyPayload {}
