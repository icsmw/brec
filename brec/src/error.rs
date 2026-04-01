use brec_common::*;
use thiserror::Error;

#[cfg(feature = "crypt")]
use crate::crypt::CryptError;
#[cfg(feature = "napi")]
use crate::napi_feature::NapiError;
#[cfg(feature = "observer")]
use crate::storage::SensorError;

/// Signature bytes that were read but did not match known block/payload signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnrecognizedSignature {
    /// Unknown block signature (fixed 4 bytes).
    Block([u8; 4]),
    /// Unknown payload signature (variable length bytes).
    Payload(Vec<u8>),
}

/// Metadata captured when an unrecognized block/payload signature is encountered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unrecognized {
    /// Unknown signature bytes and entity kind.
    pub sig: UnrecognizedSignature,
    /// Byte offset inside the current packet where signature was read (if known).
    pub pos: Option<u64>,
    /// Parsed entity length for resilient reads (if available).
    pub len: Option<u64>,
}

impl Default for Unrecognized {
    fn default() -> Self {
        Self {
            sig: UnrecognizedSignature::Block([0; 4]),
            pos: None,
            len: None,
        }
    }
}

impl Unrecognized {
    /// Creates an unrecognized block signature descriptor.
    pub fn block(sig: [u8; 4]) -> Self {
        Self {
            sig: UnrecognizedSignature::Block(sig),
            ..Self::default()
        }
    }

    /// Creates an unrecognized payload signature descriptor.
    pub fn payload(sig: Vec<u8>) -> Self {
        Self {
            sig: UnrecognizedSignature::Payload(sig),
            ..Self::default()
        }
    }

    /// Reads block signature information from a stream and returns unrecognized metadata.
    pub fn block_from<T: std::io::Read>(buf: &mut T) -> Result<Self, Error> {
        let mut sig = [0u8; BLOCK_SIG_LEN];
        let read_sig = read_exact_partial(buf, &mut sig)?;
        if read_sig < BLOCK_SIG_LEN {
            return Err(Error::NotEnoughtSignatureData(read_sig, BLOCK_SIG_LEN));
        }
        #[cfg(not(feature = "resilient"))]
        {
            Ok(Self::block(sig))
        }
        #[cfg(feature = "resilient")]
        {
            let mut unrecognized = Self::block(sig);
            let mut blk_len = [0u8; BLOCK_SIZE_FIELD_LEN];
            let read_len = read_exact_partial(buf, &mut blk_len)?;
            if read_len < BLOCK_SIZE_FIELD_LEN {
                return Err(Error::NotEnoughData(BLOCK_SIZE_FIELD_LEN - read_len));
            }
            unrecognized.len = Some(u32::from_le_bytes(blk_len) as u64);
            Ok(unrecognized)
        }
    }

    /// Parses block signature information from a byte slice.
    pub fn block_from_slice(buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < BLOCK_SIG_LEN {
            return Err(Error::NotEnoughtSignatureData(buf.len(), BLOCK_SIG_LEN));
        }
        let sig = <[u8; BLOCK_SIG_LEN]>::try_from(&buf[..BLOCK_SIG_LEN])?;
        #[cfg(not(feature = "resilient"))]
        {
            Ok(Self::block(sig))
        }
        #[cfg(feature = "resilient")]
        {
            let from = BLOCK_SIG_LEN;
            let to = BLOCK_SIG_LEN + BLOCK_SIZE_FIELD_LEN;
            if buf.len() < to {
                return Err(Error::NotEnoughData(to - buf.len()));
            }
            let blk_len = <[u8; BLOCK_SIZE_FIELD_LEN]>::try_from(&buf[from..to])?;
            let mut unrecognized = Self::block(sig);
            unrecognized.len = Some(u32::from_le_bytes(blk_len) as u64);
            Ok(unrecognized)
        }
    }

    /// Peeks and parses block signature information from a buffered reader.
    pub fn block_from_buffer<T: std::io::BufRead>(buf: &mut T) -> Result<Self, Error> {
        let bytes = buf.fill_buf()?;
        Self::block_from_slice(bytes)
    }
}

fn read_exact_partial<T: std::io::Read>(
    buf: &mut T,
    dst: &mut [u8],
) -> Result<usize, std::io::Error> {
    let mut total = 0usize;
    while total < dst.len() {
        match buf.read(&mut dst[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }
    Ok(total)
}

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
    SignatureDismatch(Unrecognized),
    /// The parsed CRC does not match the computed CRC.
    #[error("Crc doesn't match to target entity")]
    CrcDismatch,
    /// A duplicate rule of the same kind was added to a rule set.
    #[error("Same rule has been added already")]
    RuleDuplicate,
    /// A block with zero encoded length was encountered.
    #[error("Block has zero length")]
    ZeroLengthBlock,
    /// An encoded length field does not match the expected or allowed size.
    #[error("Invalid encoded length")]
    InvalidLength,
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
    #[cfg(feature = "napi")]
    /// Wrapper over `NapiError` when the `napi` feature is enabled.
    #[error("Napi: {0}")]
    Napi(#[from] NapiError),
    /// Sentinel error variant used in tests.
    #[error("Test error has been fired")]
    Test,
}

impl Error {
    /// Converts partial-read errors into [`ReadStatus::NeedMoreData`](crate::ReadStatus::NeedMoreData).
    ///
    /// Returns `Err(self)` for all non-recoverable errors.
    pub fn into_read_status<T>(self) -> Result<crate::ReadStatus<T>, Self> {
        match self {
            Self::NotEnoughtSignatureData(len, required) => {
                Ok(crate::ReadStatus::NotEnoughData((required - len) as u64))
            }
            Self::NotEnoughData(needed) => Ok(crate::ReadStatus::NotEnoughData(needed as u64)),
            err => Err(err),
        }
    }
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
