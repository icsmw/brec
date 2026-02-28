use crate::*;

/// Represents the header of a storage slot, containing metadata about the slot's capacity.
///
/// This header is used to identify and describe a fixed-size memory region (or file region)
/// that can be used for storing data chunks. It begins with a predefined signature (`STORAGE_SLOT_SIG`)
/// followed by the capacity of the slot (in bytes).
pub struct SlotHeader {
    /// Total capacity of the slot (excluding the header itself).
    pub capacity: u64,
}

impl StaticSize for SlotHeader {
    /// Returns the static size of the slot header in bytes.
    ///
    /// Includes:
    /// - 8 bytes for the signature
    /// - 8 bytes for the capacity field
    fn ssize() -> u64 {
        (STORAGE_SLOT_SIG.len() + std::mem::size_of::<u64>()) as u64
    }
}

impl ReadFrom for SlotHeader {
    /// Reads a `SlotHeader` from the provided stream.
    ///
    /// Validates the slot signature (`STORAGE_SLOT_SIG`) and reads the `capacity` field.
    ///
    /// # Errors
    /// - `Error::SignatureDismatch` if the signature is incorrect.
    /// - I/O errors if reading fails.
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != STORAGE_SLOT_SIG {
            return Err(Error::SignatureDismatch);
        }

        let mut capacity = [0u8; 8usize];
        buf.read_exact(&mut capacity)?;
        let capacity = u64::from_le_bytes(capacity);

        Ok(SlotHeader { capacity })
    }
}

impl TryReadFrom for SlotHeader {
    /// Attempts to read a `SlotHeader` from a stream with position preservation and partial read awareness.
    ///
    /// If there are not enough bytes available to read the full header, returns `ReadStatus::NotEnoughData`.
    /// On success, returns `ReadStatus::Success(SlotHeader)`.
    ///
    /// # Errors
    /// - `Error::SignatureDismatch` if the signature is invalid.
    /// - I/O errors during reading or seeking.
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < SlotHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(SlotHeader::ssize()));
        }

        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != STORAGE_SLOT_SIG {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Err(Error::SignatureDismatch);
        }

        let mut capacity = [0u8; 8usize];
        buf.read_exact(&mut capacity)?;
        let capacity = u64::from_le_bytes(capacity);

        Ok(ReadStatus::Success(SlotHeader { capacity }))
    }
}
