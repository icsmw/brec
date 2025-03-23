mod reader;
mod status;

use crate::*;
use payload::*;
pub use reader::*;
pub use status::*;

/// Trait for reading a type directly from any `Read` stream.
///
/// Used for deserializing simple binary structures.
pub trait ReadFrom {
    /// Reads and constructs the value from a binary reader.
    ///
    /// # Arguments
    /// * `buf` – A mutable reference to a type implementing `std::io::Read`.
    ///
    /// # Returns
    /// A deserialized instance of the implementing type or an error.
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Trait for reading a block-like structure directly from a byte slice.
///
/// Useful when the source is a memory buffer.
pub trait ReadBlockFromSlice {
    /// Reads the structure from a slice of bytes.
    ///
    /// # Arguments
    /// * `buf` – The source byte slice.
    /// * `skip_sig` – Whether to skip reading and verifying the signature.
    ///
    /// # Returns
    /// The deserialized block or an error if validation or decoding fails.
    fn read_from_slice<'a>(buf: &'a [u8], skip_sig: bool) -> Result<Self, Error>
    where
        Self: 'a + Sized;
}

/// Trait for reading a block-like structure from a stream.
///
/// Similar to `ReadBlockFromSlice`, but operates over a stream instead of a slice.
pub trait ReadBlockFrom {
    /// Reads the structure from a stream.
    ///
    /// # Arguments
    /// * `buf` – A stream implementing `std::io::Read`.
    /// * `skip_sig` – Whether to skip reading and verifying the signature.
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Trait for reading and validating a typed payload using a `PayloadHeader`.
///
/// Performs signature and CRC checks, and delegates to `PayloadDecode`.
pub trait ReadPayloadFrom<
    T: Sized + PayloadDecode<T> + PayloadHooks + StaticPayloadSignature + PayloadCrc,
>
{
    /// Reads and validates the payload from the stream using the provided header.
    ///
    /// # Errors
    /// Returns `SignatureDismatch` if the signature does not match,
    /// `CrcDismatch` if the checksum fails, or I/O/decoding errors.
    fn read<B: std::io::Read>(buf: &mut B, header: &PayloadHeader) -> Result<T, Error>
    where
        Self: Sized + PayloadDecode<Self> + PayloadHooks + StaticPayloadSignature,
    {
        if header.sig != T::ssig() {
            return Err(Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.payload_len()];
        buf.read_exact(&mut bytes)?;
        let value = T::decode(&bytes)?;
        let crc = value.crc()?;
        if header.crc != crc {
            return Err(Error::CrcDismatch);
        }
        Ok(value)
    }
}

/// Trait for extracting a payload of known type from a stream using a header.
///
/// Does not assume validation logic. Caller is responsible for it.
pub trait ExtractPayloadFrom<T: Sized> {
    /// Reads the payload of type `T` from the stream based on the given header.
    fn read<B: std::io::Read>(buf: &mut B, header: &PayloadHeader) -> Result<T, Error>;
}

/// Trait for attempting to read a payload if enough data is available in the stream.
///
/// Performs CRC and signature validation. Falls back gracefully if data is incomplete.
pub trait TryReadPayloadFrom<
    T: Sized
        + PayloadDecode<T>
        + PayloadHooks
        + StaticPayloadSignature
        + PayloadCrc
        + ReadPayloadFrom<T>,
>
{
    /// Attempts to read and validate the payload from a seekable stream.
    ///
    /// # Returns
    /// - `ReadStatus::Success(value)` on success.
    /// - `ReadStatus::NotEnoughData(remaining_bytes)` if more data is needed.
    fn try_read<B: std::io::Read + std::io::Seek>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error> {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < header.payload_len() as u64 {
            return Ok(ReadStatus::NotEnoughData(header.payload_len() as u64 - len));
        }
        <T as ReadPayloadFrom<T>>::read(buf, header).map(ReadStatus::Success)
    }
}

/// Trait for attempting to extract a payload with error or pending result.
///
/// Similar to `TryReadPayloadFrom`, but typically implemented manually.
pub trait TryExtractPayloadFrom<T: Sized> {
    /// Attempts to read the payload, returning status instead of panicking on incomplete data.
    fn try_read<B: std::io::Read + std::io::Seek>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error>;
}

/// Variant of `TryReadPayloadFrom` that works on buffered streams (`BufRead`).
///
/// Assumes all necessary data is already available in the buffer.
pub trait TryReadPayloadFromBuffered<
    T: Sized
        + PayloadDecode<T>
        + PayloadHooks
        + StaticPayloadSignature
        + PayloadCrc
        + ReadPayloadFrom<T>,
>
{
    /// Attempts to read and validate the payload from a buffered reader.
    ///
    /// # Returns
    /// - `ReadStatus::Success(value)` on success.
    /// - Any I/O or decoding error if decoding fails.
    fn try_read<B: std::io::BufRead>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error> {
        <T as ReadPayloadFrom<T>>::read(buf, header).map(ReadStatus::Success)
    }
}

/// Manual implementation variant of `TryReadPayloadFromBuffered`.
pub trait TryExtractPayloadFromBuffered<T: Sized> {
    /// Attempts to read the payload from a buffered reader.
    fn try_read<B: std::io::BufRead>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error>;
}

/// Generic trait for attempting to read a value from a stream with read status.
///
/// Useful for distinguishing between complete and partial data cases.
pub trait TryReadFrom {
    /// Tries to read the full structure from a seekable stream.
    ///
    /// # Returns
    /// A `ReadStatus<Self>` indicating success or how many more bytes are required.
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}

/// Variant of `TryReadFrom` for buffered, non-seekable streams.
pub trait TryReadFromBuffered {
    /// Tries to read the full structure from a buffered stream.
    ///
    /// # Returns
    /// A `ReadStatus<Self>` indicating success or failure.
    fn try_read<T: std::io::BufRead>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}
