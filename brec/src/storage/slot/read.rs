use crate::*;

impl ReadFrom for Slot {
    /// Reads a `Slot` from a stream, expecting the full structure to be available.
    ///
    /// This includes:
    /// - A `SlotHeader` (with capacity)
    /// - A sequence of `u64` length entries (`capacity` items)
    /// - A 4-byte CRC checksum
    ///
    /// # Validation
    /// After reading, the CRC is recomputed and compared with the stored one.
    ///
    /// # Errors
    /// - I/O errors during reading
    /// - `Error::SignatureDismatch` if header signature is invalid
    /// - `Error::CrcDismatch` if CRC check fails
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error> {
        let header = SlotHeader::read(buf)?;

        let mut lenghts = Vec::with_capacity(header.capacity as usize);
        for _ in 0..header.capacity {
            let mut packet_len = [0u8; 8usize];
            buf.read_exact(&mut packet_len)?;
            lenghts.push(u64::from_le_bytes(packet_len));
        }

        let mut crc = [0u8; 4usize];
        buf.read_exact(&mut crc)?;

        let slot = Slot::new(lenghts, header.capacity, crc);
        if slot.crc == slot.crc() {
            Ok(slot)
        } else {
            Err(Error::CrcDismatch)
        }
    }
}

impl TryReadFrom for Slot {
    /// Attempts to read a `Slot` from a seekable stream, with partial-read awareness.
    ///
    /// This method:
    /// - Tries to read the `SlotHeader` using `try_read`
    /// - Calculates how many bytes are required for the rest of the slot
    /// - Returns `NotEnoughData` if the stream has insufficient bytes
    /// - Performs a CRC check after reading
    ///
    /// The stream is restored to its original position if not enough data is present.
    ///
    /// # Returns
    /// - `ReadStatus::Success(slot)` on success
    /// - `ReadStatus::NotEnoughData(missing)` if full data is not yet available
    /// - `Error::CrcDismatch` if CRC check fails
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error> {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;

        let header = match SlotHeader::try_read(buf)? {
            ReadStatus::Success(header) => header,
            ReadStatus::NotEnoughData(needed) => return Ok(ReadStatus::NotEnoughData(needed)),
        };

        let needed = SlotHeader::ssize()
            + header.capacity * std::mem::size_of::<u64>() as u64
            + std::mem::size_of::<u32>() as u64;
        if len < needed {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - needed));
        }

        let mut lenghts = Vec::with_capacity(header.capacity as usize);
        for _ in 0..header.capacity {
            let mut packet_len = [0u8; 8usize];
            buf.read_exact(&mut packet_len)?;
            lenghts.push(u64::from_le_bytes(packet_len));
        }

        let mut crc = [0u8; 4usize];
        buf.read_exact(&mut crc)?;

        let slot = Slot::new(lenghts, header.capacity, crc);
        if slot.crc == slot.crc() {
            Ok(ReadStatus::Success(slot))
        } else {
            Err(Error::CrcDismatch)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CrcU32, Error, ReadFrom, ReadStatus, Slot, TryReadFrom, WriteTo};
    use std::io::{Cursor, Seek};

    fn sample_slot() -> Slot {
        let mut slot = Slot::new(vec![12, 34, 0], 3, [0; 4]);
        slot.overwrite_crc();
        slot
    }

    fn encoded_slot() -> Vec<u8> {
        let slot = sample_slot();
        let mut out = Vec::new();
        slot.write_all(&mut out).expect("slot serialization");
        out
    }

    #[test]
    fn slot_read_and_try_read_roundtrip() {
        let bytes = encoded_slot();

        let mut cursor = Cursor::new(bytes.clone());
        let read = Slot::read(&mut cursor).expect("slot read");
        assert_eq!(read.capacity, 3);
        assert_eq!(read.lenghts, vec![12, 34, 0]);
        assert_eq!(read.crc, read.crc());

        let mut cursor = Cursor::new(bytes);
        match Slot::try_read(&mut cursor).expect("slot try_read") {
            ReadStatus::Success(read) => {
                assert_eq!(read.capacity, 3);
                assert_eq!(read.lenghts, vec![12, 34, 0]);
            }
            ReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
    }

    #[test]
    fn slot_read_and_try_read_detect_crc_error() {
        let mut bytes = encoded_slot();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;

        let mut cursor = Cursor::new(bytes.clone());
        assert!(matches!(Slot::read(&mut cursor), Err(Error::CrcDismatch)));

        let mut cursor = Cursor::new(bytes);
        assert!(matches!(Slot::try_read(&mut cursor), Err(Error::CrcDismatch)));
    }

    #[test]
    fn slot_try_read_not_enough_keeps_position() {
        let bytes = encoded_slot();
        let short = bytes[..8].to_vec();
        let mut cursor = Cursor::new(short);

        match Slot::try_read(&mut cursor).expect("not enough data should not fail") {
            ReadStatus::NotEnoughData(_) => {}
            ReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }
        assert_eq!(cursor.stream_position().expect("pos"), 0);
    }
}
