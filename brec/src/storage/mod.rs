mod error;
mod iters;
mod slot;

use crate::*;
pub use error::*;
pub(crate) use iters::*;
pub use slot::*;

pub struct StorageDef<
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub slots: Vec<Slot>,
    inner: S,
    packets: PacketLocator,
    locator: FreeSlotLocator,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageDef<S, B, P, Inner>
{
    pub fn new(inner: S) -> Self {
        Self {
            slots: Vec::new(),
            inner,
            packets: PacketLocator::default(),
            locator: FreeSlotLocator::default(),
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
        }
    }
    pub fn load(&mut self) -> Result<(), StorageError> {
        let mut offset = 0;
        loop {
            self.inner.seek(std::io::SeekFrom::Start(offset))?;
            match <Slot as TryReadFrom>::try_read(&mut self.inner) {
                Ok(ReadStatus::Success(slot)) => {
                    offset += slot.size() + slot.width();
                    self.slots.push(slot);
                }
                Ok(ReadStatus::NotEnoughData(_needed)) => {
                    break;
                }
                Err(Error::CrcDismatch) => {
                    return Err(StorageError::DamagedSlot(Error::CrcDismatch))
                }
                Err(Error::SignatureDismatch) => {
                    return Err(StorageError::DamagedSlot(Error::SignatureDismatch))
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }
    pub fn insert(&mut self, mut packet: PacketDef<B, P, Inner>) -> Result<(), Error> {
        let offset = match self.locator.next(&mut self.slots) {
            Some(offset) => offset,
            None => {
                self.slots.push(Slot::default());
                self.locator
                    .next(&mut self.slots)
                    .ok_or(Error::CannotFindFreeSlot)?
            }
        };
        // Convert the packet into bytes
        let mut buffer: Vec<u8> = Vec::new();
        packet.write_all(&mut buffer)?;
        // Insert length of packet
        self.locator.insert(&mut self.slots, buffer.len() as u64)?;
        // Get update slot data
        let (slot_index, slot_offset) = self.locator.current();
        self.inner.flush()?;
        self.inner.seek(std::io::SeekFrom::Start(slot_offset))?;
        let slot = self
            .slots
            .get(slot_index)
            .ok_or(Error::CannotFindFreeSlot)?;
        // Write/Rewrite slot
        slot.write_all(&mut self.inner)?;
        self.inner.seek(std::io::SeekFrom::Start(offset))?;
        self.inner.flush()?;
        self.inner.seek(std::io::SeekFrom::Start(offset))?;
        self.inner.write_all(&buffer)?;
        self.inner.flush()?;
        Ok(())
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageDef<S, B, P, Inner>
{
    type Item = Result<ReadStatus<PacketDef<B, P, Inner>>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let location = self.packets.next(&mut self.slots)?;
        if let Err(err) = self.inner.seek(std::io::SeekFrom::Start(*location.start())) {
            return Some(Err(err.into()));
        }
        Some(<PacketDef<B, P, Inner> as TryReadFrom>::try_read(
            &mut self.inner,
        ))
    }
}
