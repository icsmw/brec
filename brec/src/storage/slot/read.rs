use crate::*;

impl ReadFrom for Slot {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
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
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
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
