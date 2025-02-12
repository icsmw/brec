#[derive(serde::Deserialize, serde::Serialize)]
struct MyPayload {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
    #[serde(skip)]
    pub __inner_buf: Vec<u8>,
}

impl brec::PayloadEncode for MyPayload {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        bincode::serialize(self)
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
        bincode::deserialize(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

impl brec::Size for MyPayload {
    fn size(&self) -> u64 {
        self.__inner_buf.len() as u64
    }
}

impl brec::PayloadCrc for MyPayload {}

impl brec::Signature for MyPayload {
    fn sig() -> brec::ByteBlock {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update("MyPayload".as_bytes());
        brec::ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl brec::ReadPayloadFrom<MyPayload> for MyPayload {}

impl brec::TryReadPayloadFrom<MyPayload> for MyPayload {}

impl brec::TryReadPayloadFromBuffered<MyPayload> for MyPayload {}

impl brec::WritePayloadTo for MyPayload {}

impl brec::WriteVectoredPayloadTo for MyPayload {}
