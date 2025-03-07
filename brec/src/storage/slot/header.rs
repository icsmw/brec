use crate::*;

pub struct SlotHeader {
    pub capacity: u64,
}

impl StaticSize for SlotHeader {
    fn ssize() -> u64 {
        (STORAGE_SLOT_SIG.len() + std::mem::size_of::<u64>()) as u64
    }
}

impl ReadFrom for SlotHeader {
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
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < SlotHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(
                PacketHeader::ssize() - SlotHeader::ssize(),
            ));
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
