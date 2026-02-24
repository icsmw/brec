#[cfg(feature = "locked_storage")]
mod locker;
#[cfg(feature = "locked_storage")]
pub use locker::{FileStorageOptions, FileWriterDef};

use crate::*;

pub struct WriterDef<
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub slots: Vec<Slot>,
    inner: S,
    locator: FreeSlotLocator,
    _phantom: std::marker::PhantomData<(B, P, Inner)>,
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > WriterDef<S, B, P, Inner>
{
    /// Creates a new storage instance with the given storage backend.
    ///
    /// # Arguments
    /// * `inner` - The storage backend implementing `Read`, `Write`, and `Seek`.
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully initialized storage.
    /// * `Err(Error)` - Failure during initialization.
    pub fn new(inner: S) -> Result<Self, Error> {
        Self {
            slots: Vec::new(),
            inner,
            locator: FreeSlotLocator::default(),
            _phantom: std::marker::PhantomData,
        }
        .load()
    }

    /// Loads storage data and initializes packet indexing.
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully loaded storage.
    /// * `Err(Error)` - Failure while loading storage.
    fn load(mut self) -> Result<Self, Error> {
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
                    return Err(Error::DamagedSlot(Box::new(Error::CrcDismatch)))
                }
                Err(Error::SignatureDismatch) => {
                    return Err(Error::DamagedSlot(Box::new(Error::SignatureDismatch)))
                }
                Err(err) => return Err(err),
            }
        }
        self.locator.setup(self.slots.iter());
        Ok(self)
    }

    /// Inserts a new packet into storage at the next available slot.
    ///
    /// # Arguments
    /// * `packet` — The `PacketDef` to be written
    ///
    /// # Returns
    /// * `Ok(())` — Packet successfully written
    /// * `Err(Error)` — If no space is found or write fails
    pub fn insert(&mut self, mut packet: PacketDef<B, P, Inner>) -> Result<(), Error> {
        let offset = match self.locator.next(&self.slots) {
            Some(offset) => offset,
            None => {
                self.slots.push(Slot::default());
                self.locator
                    .next(&self.slots)
                    .ok_or(Error::CannotFindFreeSlot)?
            }
        };
        // Convert the packet into bytes
        let mut buffer: Vec<u8> = Vec::new();
        packet.write_all(&mut buffer)?;
        // Insert length of packet
        self.locator.insert(&mut self.slots, buffer.len() as u64)?;
        // Get updated slot data
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
