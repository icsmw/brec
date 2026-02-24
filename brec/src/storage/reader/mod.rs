mod iters;

use crate::*;
pub(crate) use iters::*;

pub type NthFilteredPacket<B, P, Inner> = Option<LookInStatus<PacketDef<B, P, Inner>>>;

pub struct ReaderDef<
    S: std::io::Read + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub slots: Vec<AnchoredSlot>,
    inner: S,
    locator: FreeSlotLocator,
    rules: RulesDef<B, BR, P, Inner>,
}

impl<
        S: std::io::Read + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > ReaderDef<S, B, BR, P, Inner>
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
                    let position = offset;
                    offset += slot.size() + slot.width();
                    self.slots.push(AnchoredSlot::new(slot, position));
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
        self.locator
            .setup(self.slots.iter().map(|anchored| &anchored.inner));
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

    /// Returns the number of records currently stored.
    pub fn count(&self) -> usize {
        let (slot_index, _) = self.locator.current();
        let Some(slot) = self.slots.get(slot_index) else {
            return self.slots.len() * DEFAULT_SLOT_CAPACITY;
        };
        let Some(index) = slot.get_free_slot_index() else {
            return self.slots.len() * DEFAULT_SLOT_CAPACITY;
        };
        slot_index * DEFAULT_SLOT_CAPACITY + index
    }

    /// Returns an iterator over all packets in the storage (no filtering).
    ///
    /// # Returns
    /// * `ReaderIterator` yielding `Result<PacketDef<..>, Error>`
    pub fn iter<'a>(
        &'a mut self,
    ) -> ReaderIterator<'a, impl Iterator<Item = &'a Slot>, S, B, P, Inner> {
        ReaderIterator::new(
            &mut self.inner,
            self.slots.iter().map(|anchored| &anchored.inner),
        )
    }

    /// Returns a filtered iterator over packets using configured rules.
    ///
    /// # Returns
    /// * `ReaderFilteredIterator` yielding packets that pass rules
    pub fn filtered<'a>(
        &'a mut self,
    ) -> ReaderFilteredIterator<'a, impl Iterator<Item = &'a Slot>, S, B, BR, P, Inner> {
        ReaderFilteredIterator::new(
            &mut self.inner,
            self.slots.iter().map(|anchored| &anchored.inner),
            &self.rules,
        )
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
    /// * `ReaderRangeIterator` over the given range
    pub fn range(
        &mut self,
        from: usize,
        len: usize,
    ) -> ReaderRangeIterator<'_, S, B, BR, P, Inner> {
        ReaderRangeIterator::new(self, from, len)
    }

    /// Returns a filtered range iterator applying rules to each packet.
    ///
    /// # Arguments
    /// * `from` — Starting index
    /// * `len` — Number of packets to yield
    ///
    /// # Returns
    /// * `ReaderRangeFilteredIterator` that yields only accepted packets
    pub fn range_filtered(
        &mut self,
        from: usize,
        len: usize,
    ) -> ReaderRangeFilteredIterator<'_, S, B, BR, P, Inner> {
        ReaderRangeFilteredIterator::new(self, from, len)
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
