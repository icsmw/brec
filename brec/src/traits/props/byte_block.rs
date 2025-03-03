use crate::*;

#[derive(PartialEq, Eq, Debug)]
pub enum ByteBlock {
    Len4([u8; 4]),
    Len8([u8; 8]),
    Len16([u8; 16]),
    Len32([u8; 32]),
    Len64([u8; 64]),
    Len128([u8; 128]),
}

impl ByteBlock {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            ByteBlock::Len4(arr) => arr,
            ByteBlock::Len8(arr) => arr,
            ByteBlock::Len16(arr) => arr,
            ByteBlock::Len32(arr) => arr,
            ByteBlock::Len64(arr) => arr,
            ByteBlock::Len128(arr) => arr,
        }
    }
    pub fn size(&self) -> usize {
        self.as_slice().len()
    }
    pub fn is_valid_capacity(cap: u8) -> Result<(), Error> {
        if [4, 8, 16, 32, 64, 128].contains(&cap) {
            Ok(())
        } else {
            Err(Error::InvalidCapacity(
                cap as usize,
                "4, 8, 16, 32, 64, 128".to_string(),
            ))
        }
    }
}

impl TryFrom<Vec<u8>> for ByteBlock {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.len() {
            4 => value.try_into().map(Self::Len4),
            8 => value.try_into().map(Self::Len8),
            16 => value.try_into().map(Self::Len16),
            32 => value.try_into().map(Self::Len32),
            64 => value.try_into().map(Self::Len64),
            128 => value.try_into().map(Self::Len128),
            invalid => {
                return Err(Error::InvalidCapacity(
                    invalid,
                    "4, 8, 16, 32, 64, 128".to_string(),
                ))
            }
        }
        .map_err(|_| Error::FailExtractByteBlock)
    }
}
