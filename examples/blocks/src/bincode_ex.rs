#[derive(serde::Deserialize, serde::Serialize)]
struct MyPayload {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
    #[serde(skip)]
    pub __inner_buf: Vec<u8>,
}

impl MyPayload {
    pub fn encode(&mut self) -> std::io::Result<()> {
        self.__inner_buf = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(())
    }

    pub fn decode(buf: &[u8]) -> Result<Self, brec::Error> {
        bincode::deserialize(buf).map_err(|e| brec::Error::EncodeError(e.to_string()))
    }
}

impl brec::Size for MyPayload {
    fn size(&self) -> u64 {
        self.__inner_buf.len() as u64
    }
}

impl brec::Crc for MyPayload {
    fn crc(&self) -> brec::ByteBlock {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.__inner_buf);
        brec::ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl brec::Signature for MyPayload {
    fn sig() -> brec::ByteBlock {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update("MyPayload".as_bytes());
        brec::ByteBlock::Len4(hasher.finalize().to_le_bytes())
    }
}

impl brec::ReadPayloadFrom for MyPayload {
    fn read<T: std::io::Read>(
        buf: &mut T,
        header: &brec::PayloadHeader,
    ) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if header.sig != MyPayload::sig() {
            return Err(brec::Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.payload_len()];
        buf.read_exact(&mut bytes)?;
        if header.crc != bytes.crc() {
            return Err(brec::Error::CrcDismatch);
        }
        MyPayload::decode(&bytes)
    }
}

impl brec::TryReadPayloadFrom for MyPayload {}

impl brec::TryReadPayloadFromBuffered for MyPayload {}

impl brec::MutWriteTo for MyPayload {
    fn write<T: std::io::Write>(&mut self, writer: &mut T) -> std::io::Result<usize> {
        self.encode()?;
        let mut header = [0u8; brec::PayloadHeader::LEN];
        brec::PayloadHeader::write(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write(&self.__inner_buf)
    }
    fn write_all<T: std::io::Write>(&mut self, writer: &mut T) -> std::io::Result<()> {
        self.encode()?;
        let mut header = [0u8; brec::PayloadHeader::LEN];
        brec::PayloadHeader::write(self, &mut header)?;
        writer.write_all(&header)?;
        writer.write_all(&self.__inner_buf)
    }
}

impl brec::MutWriteVectoredTo for MyPayload {
    fn slices(&mut self) -> std::io::Result<brec::IoSlices> {
        self.encode()?;
        let mut slices = brec::IoSlices::default();
        let mut header = [0u8; brec::PayloadHeader::LEN];
        brec::PayloadHeader::write(self, &mut header)?;
        slices.add_buffered(header.to_vec());
        slices.add_slice(&self.__inner_buf);
        Ok(slices)
    }
}
