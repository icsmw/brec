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
    /// Creates a new reader instance with the given storage backend.
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
                    return Err(Error::DamagedSlot(Box::new(Error::CrcDismatch)));
                }
                Err(Error::SignatureDismatch) => {
                    return Err(Error::DamagedSlot(Box::new(Error::SignatureDismatch)));
                }
                Err(err) => return Err(err),
            }
        }
        self.locator
            .setup(self.slots.iter().map(|anchored| &anchored.inner));
        Ok(self)
    }

    pub fn reload(&mut self) -> Result<usize, Error> {
        let mut source_pos;

        let last = match self.slots.last().map(|v| (v, v.inner.expand())) {
            Some((last, (Some(offset), Some(index), crc))) => {
                source_pos = last.offset;
                Some((offset, index, crc))
            }
            Some((last, (None, None, _))) => {
                // Slot is full, because no free offset or/and index
                source_pos = last.offset + last.inner.width() + last.inner.size();
                None
            }
            _ => {
                // No slots
                source_pos = 0;
                None
            }
        };
        let count = self.slots.len();
        loop {
            self.inner.seek(std::io::SeekFrom::Start(source_pos))?;
            match <Slot as TryReadFrom>::try_read(&mut self.inner) {
                Ok(ReadStatus::Success(slot)) => {
                    if let Some((_, _, crc)) = last
                        && self.slots.len() == count
                    {
                        if crc == slot.crc {
                            return Ok(0);
                        }
                        if let Some(lst) = self.slots.get_mut(count.saturating_sub(1)) {
                            lst.inner = slot;
                            if lst.inner.get_free_slot_index().is_none() {
                                // Slot is full, move source position to the end of this slot
                                source_pos += lst.inner.size() + lst.inner.width();
                            } else {
                                // Slot has free space, so we can stop here
                                break;
                            }
                        } else {
                            return Err(Error::AccessSlot(count.saturating_sub(1)));
                        }
                    } else {
                        let position = source_pos;
                        source_pos += slot.size() + slot.width();
                        self.slots.push(AnchoredSlot::new(slot, position));
                    }
                }
                Ok(ReadStatus::NotEnoughData(needed)) => {
                    match (last.is_none(), self.slots.len() == count) {
                        (true, true) => {
                            return Ok(0);
                        }
                        (false, true) => {
                            if needed == SlotHeader::ssize() {
                                // No space in last slot, no slot after
                                break;
                            }
                            // Cannot read again last slot
                            return Err(Error::DamagedSlot(Box::new(Error::NotEnoughData(
                                needed as usize,
                            ))));
                        }
                        (false, false) | (true, false) => break,
                    }
                }
                Err(Error::CrcDismatch) => {
                    return Err(Error::DamagedSlot(Box::new(Error::CrcDismatch)));
                }
                Err(Error::SignatureDismatch) => {
                    return Err(Error::DamagedSlot(Box::new(Error::SignatureDismatch)));
                }
                Err(err) => return Err(err),
            }
        }

        let read = match (last, self.slots.len() == count, self.slots.last()) {
            (Some((_, index, _)), true, Some(last)) => {
                // Last slot has been update, no new slot has been added
                if let Some(idx) = last.inner.get_free_slot_index() {
                    idx.saturating_sub(index)
                } else {
                    DEFAULT_SLOT_CAPACITY.saturating_sub(index)
                }
            }
            (Some((_, index, _)), false, Some(last)) => {
                // Last slot has been updated, but no new slot has been added
                DEFAULT_SLOT_CAPACITY.saturating_sub(index)
                    + last.inner.get_free_slot_index().unwrap_or(0)
                    + self.slots.len().saturating_sub(count) * DEFAULT_SLOT_CAPACITY
            }
            (None, false, Some(last)) => {
                // In cases:
                // - No last slot, but new slots have been added
                // - Last slot is full, but new slots have been added
                last.inner.get_free_slot_index().unwrap_or(0)
                    + self.slots.len().saturating_sub(count + 1) * DEFAULT_SLOT_CAPACITY
            }
            _ => 0,
        };
        self.locator
            .setup(self.slots.iter().map(|anchored| &anchored.inner));
        Ok(read)
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
        // TODO: try to get rid of locator

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

    pub fn seek<'a>(
        &'a mut self,
        packet: usize,
    ) -> Result<ReaderIterator<'a, impl Iterator<Item = &'a Slot>, S, B, P, Inner>, Error> {
        ReaderIterator::new(
            &mut self.inner,
            self.slots.iter().map(|anchored| &anchored.inner),
        )
        .seek(packet)
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
