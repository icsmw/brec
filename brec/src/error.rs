use thiserror::Error;

#[cfg(feature = "crypt")]
use crate::crypt::CryptError;
#[cfg(feature = "observer")]
use crate::storage::SensorError;

/// Unified error type used by `brec` APIs.
#[derive(Error, Debug)]
pub enum Error {
    /// The source did not contain enough data to complete the requested operation.
    #[error("Not enought data; required = {0}")]
    NotEnoughData(usize),
    /// There was not enough data to read a complete signature.
    #[error("Not enought data to read signature; data len = {0}; required = {1}")]
    NotEnoughtSignatureData(usize, usize),
    /// The input bytes were not aligned as required by the target type.
    #[error("Invalid data align; data len = {0}; required = {1}; offset = {2} (expected 0)")]
    InvalidAlign(usize, usize, usize),
    /// A caller-provided buffer had an unexpected capacity.
    #[error("Invalid buffer capacity: {0}; expected: {1}")]
    InvalidCapacity(usize, String),
    /// Conversion from a raw slice into a fixed-size array failed.
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    /// The parsed signature does not match the expected entity type.
    #[error("Signature doesn't match to target entity")]
    SignatureDismatch,
    /// The parsed CRC does not match the computed CRC.
    #[error("Crc doesn't match to target entity")]
    CrcDismatch,
    /// A duplicate rule of the same kind was added to a rule set.
    #[error("Same rule has been added already")]
    RuleDuplicate,
    /// A block with zero encoded length was encountered.
    #[error("Block has zero length")]
    ZeroLengthBlock,
    /// The packet contains more blocks than `brec` allows.
    #[error("Attempt to read more blocks than allowed")]
    MaxBlocksCount,
    /// A slice pointer is misaligned for the target block type.
    #[error("Misaligned slice pointer")]
    MisalignedPointer,
    /// A slice length does not match the expected binary layout.
    #[error("Unexpected slice length")]
    UnexpectedSliceLength,
    /// A generic conversion failed and includes both source and error text.
    #[error("Fail converting \"{0}\" with error: {1}")]
    FailedConverting(String, String),
    /// Wrapper over `std::io::Error`.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    /// `ByteBlock` extraction from a vector-backed buffer failed.
    #[error("Fail to exctract data from vector for ByteBlock")]
    FailExtractByteBlock,
    /// A payload header could not be read from the source.
    #[error("Fail to read payload header")]
    FailToReadPayloadHeader,
    /// Allocation of a temporary buffer failed.
    #[error("Memory allocation failed")]
    MemoryAllocationFailed,
    /// Payload encoding failed.
    #[error("Encoding error: {0}")]
    EncodeError(String),
    /// The reader has no pending packet to accept.
    #[error("No pending packet to accept")]
    NoPendingPacket,
    /// A packet header could not be read from the source.
    #[error("Fail to read packet header")]
    FailToReadPacketHeader,
    /// Internal packet reader state became inconsistent.
    #[error("PacketBufReader fall down into invalid logic")]
    InvalidPacketReaderLogic,
    /// No suitable free slot could be located in storage.
    #[error("Fail to find free slot")]
    CannotFindFreeSlot,
    /// A slot exists but no suitable free area inside it could be found.
    #[error("Fail to find free palce in slot")]
    CannotFindFreePlaceInSlot,
    /// Insertion into a storage slot failed.
    #[error("Fail to insert data into slot")]
    CannotInsertIntoSlot,
    /// A storage slot is damaged; the nested error describes the reason.
    #[error("Damaged slot: {0}")]
    DamagedSlot(Box<Error>),
    /// Reading blocks retried too many times without converging.
    #[error("Too many attempts to read block; made {0} attempts")]
    TooManyAttemptsToReadBlock(usize),
    /// An index or offset exceeded the valid bounds.
    #[error("Out of bounds; len = {0}; requested = {1}")]
    OutOfBounds(usize, usize),
    /// A path expected to be a file is not a regular file.
    #[error("Path isn't a file: {0}")]
    PathIsNotFile(String),
    /// A locked storage file cannot currently be opened for writing.
    #[error("File is locked: {0}")]
    FileIsLocked(String),
    /// Waiting for a locked file timed out.
    #[error("Timeout error. File is locked: {0}")]
    TimeoutToWaitLockedFile(String),
    /// Acquiring a file lock failed.
    #[error("Fail to lock file: {0}")]
    FailToLockFile(std::io::Error),
    /// Access to a storage slot by index failed.
    #[error("Fail to access slot:{0}")]
    AccessSlot(usize),
    /// The source contained no readable data.
    #[error("Empty source")]
    EmptySource,
    #[cfg(feature = "crypt")]
    /// Wrapper over `CryptError` when the `crypt` feature is enabled.
    #[error("Crypt: {0}")]
    Crypt(#[from] CryptError),
    #[cfg(feature = "observer")]
    /// Wrapper over storage observer sensor errors.
    #[error("Sensor: {0}")]
    Sensor(SensorError),
    #[cfg(feature = "observer")]
    /// Observer setup was attempted without a subscription.
    #[error("No subscription")]
    NoSubscription,
    /// Sentinel error variant used in tests.
    #[error("Test error has been fired")]
    Test,
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::other(value.to_string())
    }
}

#[cfg(feature = "observer")]
impl From<SensorError> for Error {
    fn from(value: SensorError) -> Self {
        Error::Sensor(value)
    }
}
