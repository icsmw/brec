use crate::*;

pub struct TestBlock {
    #[allow(dead_code)]
    field: u8,
}

impl Size for TestBlock {
    fn size(&self) -> u64 {
        0
    }
}

impl WriteVectoredTo for TestBlock {
    fn slices(&self) -> std::io::Result<IoSlices<'_>> {
        Err(std::io::Error::other("test"))
    }
}

impl WriteTo for TestBlock {
    fn write<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<usize> {
        Err(std::io::Error::other("test"))
    }
    fn write_all<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<()> {
        Err(std::io::Error::other("test"))
    }
}

impl TryReadFromBuffered for TestBlock {
    fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        Err(Error::Test)
    }
}

impl TryReadFrom for TestBlock {
    fn try_read<T: std::io::Read + std::io::Seek>(_: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        Err(Error::Test)
    }
}

impl ReadFrom for TestBlock {
    fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Err(Error::Test)
    }
}

impl ReadBlockFrom for TestBlock {
    fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Err(Error::Test)
    }
}

impl ReadBlockFromSlice for TestBlock {
    fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
    where
        Self: 'a + Sized,
    {
        Err(Error::Test)
    }
}

impl BlockDef for TestBlock {}

impl BlockReferredDef<TestBlock> for TestBlock {}

pub struct TestPayload {
    #[allow(dead_code)]
    field: u8,
}

impl WriteVectoredMutTo for TestPayload {
    fn slices(&mut self) -> std::io::Result<IoSlices<'_>> {
        Err(std::io::Error::other("test"))
    }
}

impl WriteMutTo for TestPayload {
    fn write<T: std::io::Write>(&mut self, _: &mut T) -> std::io::Result<usize> {
        Err(std::io::Error::other("test"))
    }
    fn write_all<T: std::io::Write>(&mut self, _: &mut T) -> std::io::Result<()> {
        Err(std::io::Error::other("test"))
    }
}

impl PayloadSignature for TestPayload {
    fn sig(&self) -> ByteBlock {
        ByteBlock::Len4([0, 0, 0, 0])
    }
}

impl PayloadEncodeReferred for TestPayload {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Err(std::io::Error::other("test"))
    }
}

impl PayloadHooks for TestPayload {
    fn after_decode(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("test"))
    }
    fn before_encode(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("test"))
    }
}

impl PayloadEncode for TestPayload {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::other("test"))
    }
}

impl PayloadCrc for TestPayload {
    fn crc(&self) -> std::io::Result<ByteBlock> {
        Err(std::io::Error::other("test"))
    }
    fn crc_size() -> usize {
        0
    }
}

impl PayloadSize for TestPayload {
    fn size(&self) -> std::io::Result<u64> {
        Err(std::io::Error::other("test"))
    }
}
impl PayloadInnerDef for TestPayload {}

impl TryExtractPayloadFromBuffered<TestPayload> for TestPayload {
    fn try_read<B: std::io::BufRead>(
        _: &mut B,
        _: &PayloadHeader,
    ) -> Result<ReadStatus<TestPayload>, Error> {
        Err(Error::Test)
    }
}

impl TryExtractPayloadFrom<TestPayload> for TestPayload {
    fn try_read<B: std::io::Read + std::io::Seek>(
        _: &mut B,
        _: &PayloadHeader,
    ) -> Result<ReadStatus<TestPayload>, Error> {
        Err(Error::Test)
    }
}

impl ExtractPayloadFrom<TestPayload> for TestPayload {
    fn read<B: std::io::Read>(_: &mut B, _: &PayloadHeader) -> Result<TestPayload, Error> {
        Err(Error::Test)
    }
}

impl PayloadDef<TestPayload> for TestPayload {}
