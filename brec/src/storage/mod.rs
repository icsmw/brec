mod iters;
mod locator;
mod range;
mod slot;

use crate::*;
pub(crate) use iters::*;
pub(crate) use locator::*;
pub(crate) use slot::*;

pub type NthFilteredPacket<B, P, Inner> = Option<LookInStatus<PacketDef<B, P, Inner>>>;

/// `StorageDef` serves as a storage management component. It enables storing and retrieving packets in various ways.
/// Since packets are stored using slots, `StorageDef` provides fast access to a packet by its sequential index.
/// This is particularly useful when reading packets as frames, as retrieving a specific range of packets
/// (e.g., 100..150 or 500..600) is optimized. `StorageDef` maintains packet positions in `Slot` headers,
/// an internal structure within `StorageDef`.
///
/// ## Capabilities of `StorageDef`
///
/// - Storing `Packet` instances in the storage backend
/// - Sequentially reading `Packet` instances from storage
/// - Reading `Packet` instances with prefiltering and filtering
/// - Reading `Packet` instances within a specified range
///
/// ## Filtering Mechanism
/// `StorageDef` supports multiple filtering methods when reading packets:
/// - **Filtering by `Block` (Prefiltering):** Once a packet is detected and all its `Block` components are processed,
///   a filter can determine whether to proceed with parsing or skip the packet entirely. This filter enables
///   high-performance traversal since `Payload` parsing is bypassed, delivering only packets with relevant blocks.
/// - **Filtering by `Packet`:** This filter is applied after the `Packet` is fully parsed. It can be used, for instance,
///   to search within each packet's `Payload`.
/// - **Combined Filtering:** Both `Block` and `Packet` filters are applied together.
///
/// ## Note
/// There is no need to use `StorageDef` directly or specify all generic parameters manually. When invoking
/// `brec::include_generated!()`, a shorter variant, `Storage<S>`, is generated with a single generic parameter
/// defining the storage source.
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

    /// Adds a processing rule. See `RuleDef` for more details.
    pub fn add_rule(&mut self, rule: RuleDef<B, BR, P, Inner>) -> Result<(), Error> {
        self.rules.add_rule(rule)
    }

    /// Removes a previously added rule. See `RuleDef` for more details.
    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules.remove_rule(rule);
    }

    /// Inserts a `Packet` at the end of the storage.
    ///
    /// # Arguments
    /// * `packet` - The packet to be inserted.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully inserted the packet.
    /// * `Err(Error)` - Failure during insertion.
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

    /// Returns an iterator over all `Packet` instances in the storage.
    pub fn iter(&mut self) -> StorageIterator<'_, S, B, P, Inner> {
        StorageIterator::new(&mut self.inner, &self.slots)
    }

    pub fn filtered(&mut self) -> StorageFilteredIterator<'_, S, B, BR, P, Inner> {
        StorageFilteredIterator::new(&mut self.inner, &self.slots, &self.rules)
    }

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

    pub fn range(
        &mut self,
        from: usize,
        len: usize,
    ) -> StorageRangeIterator<'_, S, B, BR, P, Inner> {
        StorageRangeIterator::new(self, from, len)
    }

    pub fn range_filtered(
        &mut self,
        from: usize,
        len: usize,
    ) -> StorageRangeFilteredIterator<'_, S, B, BR, P, Inner> {
        StorageRangeFilteredIterator::new(self, from, len)
    }

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
