use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not enought data; required = {0}")]
    NotEnoughData(usize),
    #[error("Not enought data to read signature; data len = {0}; required = {1}")]
    NotEnoughtSignatureData(usize, usize),
    #[error("Invalid data align; data len = {0}; required = {1}; offset = {2} (expected 0)")]
    InvalidAlign(usize, usize, usize),
    #[error("Invalid buffer capacity: {0}; expected: {1}")]
    InvalidCapacity(usize, String),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Signature doesn't match to target entity")]
    SignatureDismatch,
    #[error("Crc doesn't match to target entity")]
    CrcDismatch,
    #[error("Same rule has been added already")]
    RuleDuplicate,
    #[error("Block has zero length")]
    ZeroLengthBlock,
    #[error("Attempt to read more blocks than allowed")]
    MaxBlocksCount,
    #[error("Misaligned slice pointer")]
    MisalignedPointer,
    #[error("Unexpected slice length")]
    UnexpectedSliceLength,
    #[error("Fail converting \"{0}\" with error: {1}")]
    FailedConverting(String, String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Fail to exctract data from vector for ByteBlock")]
    FailExtractByteBlock,
    #[error("Fail to read payload header")]
    FailToReadPayloadHeader,
    #[error("Memory allocation failed")]
    MemoryAllocationFailed,
    #[error("Encoding error: {0}")]
    EncodeError(String),
    #[error("No pending packet to accept")]
    NoPendingPacket,
}
