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
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error> {
        let mut sig = [0u8; 8];
        buf.read_exact(&mut sig)?;
        if sig != STORAGE_SLOT_SIG {
            return Err(Error::SignatureDismatch(Unrecognized::payload(sig.to_vec())));
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
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error> {
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
            return Err(Error::SignatureDismatch(Unrecognized::payload(sig.to_vec())));
        }

        let mut capacity = [0u8; 8usize];
        buf.read_exact(&mut capacity)?;
        let capacity = u64::from_le_bytes(capacity);

        Ok(ReadStatus::Success(SlotHeader { capacity }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, ReadFrom, ReadStatus, SlotHeader, StaticSize, TryReadFrom, WriteTo};
    use std::io::{Cursor, Seek, SeekFrom};

    fn encoded_header(capacity: u64) -> Vec<u8> {
        let slot = crate::Slot::new(vec![0; capacity as usize], capacity, [0; 4]);
        let mut out = Vec::new();
        slot.write_all(&mut out).expect("slot serialization");
        out[..SlotHeader::ssize() as usize].to_vec()
    }

    #[test]
    fn slot_header_read_and_try_read_success() {
        let bytes = encoded_header(7);

        let mut cursor = Cursor::new(bytes.clone());
        let header = SlotHeader::read(&mut cursor).expect("read slot header");
        assert_eq!(header.capacity, 7);

        let mut cursor = Cursor::new(bytes);
        match SlotHeader::try_read(&mut cursor).expect("try_read slot header") {
            ReadStatus::Success(h) => assert_eq!(h.capacity, 7),
            ReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
    }

    #[test]
    fn slot_header_try_read_not_enough_and_bad_signature() {
        let short = vec![1_u8, 2, 3];
        let mut cursor = Cursor::new(short);
        match SlotHeader::try_read(&mut cursor).expect("short input should not fail") {
            ReadStatus::NotEnoughData(need) => assert_eq!(need, SlotHeader::ssize()),
            ReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }
        assert_eq!(cursor.stream_position().expect("pos"), 0);

        let mut bad = encoded_header(3);
        bad[0] ^= 0xFF;
        let mut cursor = Cursor::new(bad);
        assert!(matches!(
            SlotHeader::try_read(&mut cursor),
            Err(Error::SignatureDismatch(_))
        ));
        assert_eq!(
            cursor.seek(SeekFrom::Current(0)).expect("pos"),
            0,
            "position should be restored on signature mismatch"
        );
    }
}
