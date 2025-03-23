mod iters;
mod locator;
mod slot;

use crate::*;
pub(crate) use iters::*;
pub(crate) use locator::*;
pub(crate) use slot::*;

pub type NthFilteredPacket<B, P, Inner> = Option<LookInStatus<PacketDef<B, P, Inner>>>;

/// Represents persistent binary storage for brec packets, backed by an abstract I/O stream.
///
/// `StorageDef` manages serialized `PacketDef` instances, organized into fixed-capacity `Slot`s.
/// Each slot contains lengths of up to 500 packets and their relative positions within the file.
/// This structure enables fast indexed access to packets by computing their exact offsets without
/// scanning the entire file.
///
/// Each `Slot` also includes a CRC checksum, ensuring integrity of the slot's metadata. As long as
/// slots and packet data remain intact, reading is fast and reliable.
///
/// If the file becomes corrupted (e.g., a broken slot header or damaged packet), reading
/// via `StorageDef` will fail entirely, since indexing relies on precise structure.
/// However, this is not a critical limitation — to recover packets from a damaged file,
/// you can use [`PacketBufReaderDef`](crate::PacketBufReaderDef), which performs a tolerant scan of
/// the file and extracts valid packets, ignoring corrupted data or garbage.
///
/// # Type Parameters
/// - `S`: Backend stream implementing `Read + Write + Seek` (e.g. file, memory buffer)
/// - `B`: Block type implementing [`BlockDef`](crate::BlockDef)
/// - `BR`: Block reference type implementing [`BlockReferredDef`](crate::BlockReferredDef)
/// - `P`: Payload wrapper implementing [`PayloadDef`](crate::PayloadDef)
/// - `Inner`: Inner payload object implementing [`PayloadInnerDef`](crate::PayloadInnerDef)
///
/// # Notes
///
/// - One `Slot` is created for every 500 packets (`DEFAULT_SLOT_CAPACITY = 500`).  
/// - Each slot tracks the lengths of written packets, allowing the system to calculate offsets without parsing data.  
/// - All writes (including `Slot` metadata) are flushed immediately, enabling crash-safe appends.
/// - This storage is **append-only**; there's no support for removal or in-place overwrite.
///
/// ## Short type alias
///
/// There is no need to use `StorageDef` directly or specify all generic parameters manually.  
/// When invoking `brec::include_generated!()`, a short alias `Storage<S>` is generated,  
/// requiring only the stream type `S`. This is the recommended way to use the storage in real projects.
///
/// ## Example
/// ```no_run
/// brec::include_generated!();
///
/// use std::fs::OpenOptions;
/// let file = OpenOptions::new().read(true).write(true).open("log.brec")?;
/// let mut storage = Storage::new(file)?;
/// for pkt in storage.iter() {
///     println!("{:?}", pkt?);
/// }
/// ```
pub struct StorageDef<
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub slots: Vec<Slot>,
    inner: S,
    locator: FreeSlotLocator,
    rules: RulesDef<B, BR, P, Inner>,
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageDef<S, B, BR, P, Inner>
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
            rules: RulesDef::default(),
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
        Ok(self)
    }

    /// Adds a packet filter or processing rule.
    ///
    /// # Arguments
    /// * `rule` — A new rule to apply (see `RuleDef`)
    ///
    /// # Returns
    /// * `Ok(())` — Rule added successfully
    /// * `Err(Error::RuleDuplicate)` — Rule of the same type already exists
    pub fn add_rule(&mut self, rule: RuleDef<B, BR, P, Inner>) -> Result<(), Error> {
        self.rules.add_rule(rule)
    }

    /// Removes a previously added rule by its identifier.
    ///
    /// # Arguments
    /// * `rule` — Identifier of the rule to remove (`RuleDefId`)
    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules.remove_rule(rule);
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

    /// Returns an iterator over all packets in the storage (no filtering).
    ///
    /// # Returns
    /// * `StorageIterator` yielding `Result<PacketDef<..>, Error>`
    pub fn iter(&mut self) -> StorageIterator<'_, S, B, P, Inner> {
        StorageIterator::new(&mut self.inner, &self.slots)
    }

    /// Returns a filtered iterator over packets using configured rules.
    ///
    /// # Returns
    /// * `StorageFilteredIterator` yielding packets that pass rules
    pub fn filtered(&mut self) -> StorageFilteredIterator<'_, S, B, BR, P, Inner> {
        StorageFilteredIterator::new(&mut self.inner, &self.slots, &self.rules)
    }

    /// Retrieves the `nth` packet by global index (across all slots).
    ///
    /// # Arguments
    /// * `nth` — Zero-based index of the packet
    ///
    /// # Returns
    /// * `Ok(Some(PacketDef))` — Packet found
    /// * `Ok(None)` — No packet exists at this index
    /// * `Err(Error)` — On slot mismatch, CRC failure, or I/O error
    pub fn nth(&mut self, nth: usize) -> Result<Option<PacketDef<B, P, Inner>>, Error> {
        let slot_index = nth / DEFAULT_SLOT_CAPACITY;
        let index_in_slot = nth % DEFAULT_SLOT_CAPACITY;
        let Some(slot) = self.slots.get(slot_index) else {
            return Ok(None);
        };
        if slot.is_empty(index_in_slot)? {
            return Ok(None);
        }
        let Some(mut offset) = slot.get_slot_offset(index_in_slot) else {
            return Ok(None);
        };
        offset += self.slots[..slot_index]
            .iter()
            .map(|slot| slot.width() + slot.size())
            .sum::<u64>();
        self.inner.seek(std::io::SeekFrom::Start(offset))?;
        match <PacketDef<B, P, Inner> as TryReadFrom>::try_read(&mut self.inner)? {
            ReadStatus::Success(pkg) => Ok(Some(pkg)),
            ReadStatus::NotEnoughData(needed) => Err(Error::NotEnoughData(needed as usize)),
        }
    }

    /// Returns an iterator over a specific range of packets by global index.
    ///
    /// # Arguments
    /// * `from` — Starting index (inclusive)
    /// * `len` — Number of packets to iterate
    ///
    /// # Returns
    /// * `StorageRangeIterator` over the given range
    pub fn range(
        &mut self,
        from: usize,
        len: usize,
    ) -> StorageRangeIterator<'_, S, B, BR, P, Inner> {
        StorageRangeIterator::new(self, from, len)
    }

    /// Returns a filtered range iterator applying rules to each packet.
    ///
    /// # Arguments
    /// * `from` — Starting index
    /// * `len` — Number of packets to yield
    ///
    /// # Returns
    /// * `StorageRangeFilteredIterator` that yields only accepted packets
    pub fn range_filtered(
        &mut self,
        from: usize,
        len: usize,
    ) -> StorageRangeFilteredIterator<'_, S, B, BR, P, Inner> {
        StorageRangeFilteredIterator::new(self, from, len)
    }

    /// Returns the filtered result of the `nth` packet.
    ///
    /// This method applies all configured rules (block, payload, full packet).
    ///
    /// # Arguments
    /// * `from` — Index of the packet to filter
    ///
    /// # Returns
    /// * `Ok(Some(LookInStatus::Accepted(size, packet)))` — Passed all filters
    /// * `Ok(Some(LookInStatus::Denied(size)))` — Filtered out
    /// * `Ok(Some(LookInStatus::NotEnoughData(n)))` — Incomplete
    /// * `Ok(None)` — No packet at index
    /// * `Err(Error)` — On I/O or parse failure
    pub(crate) fn nth_filtered(
        &mut self,
        from: usize,
    ) -> Result<NthFilteredPacket<B, P, Inner>, Error> {
        let slot_index = from / DEFAULT_SLOT_CAPACITY;
        let index_in_slot = from % DEFAULT_SLOT_CAPACITY;
        let Some(slot) = self.slots.get(slot_index) else {
            return Ok(None);
        };
        if slot.is_empty(index_in_slot)? {
            return Ok(None);
        }
        let Some(mut offset) = slot.get_slot_offset(index_in_slot) else {
            return Ok(None);
        };
        offset += self.slots[..slot_index]
            .iter()
            .map(|slot| slot.width() + slot.size())
            .sum::<u64>();
        self.inner.seek(std::io::SeekFrom::Start(offset))?;
        match PacketDef::filtered(&mut self.inner, &self.rules)? {
            LookInStatus::Accepted(size, pkg) => Ok(Some(LookInStatus::Accepted(size, pkg))),
            LookInStatus::Denied(size) => Ok(Some(LookInStatus::Denied(size))),
            LookInStatus::NotEnoughData(needed) => Err(Error::NotEnoughData(needed)),
        }
    }
}
