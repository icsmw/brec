mod iters;
mod locator;
mod range;
mod slot;

use std::ops::RangeInclusive;

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
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub slots: Vec<Slot>,
    inner: S,
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
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
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

    /// Returns an iterator over `Packet` instances in storage with filtering based on `Block` data.
    ///
    /// This filtering method improves performance by skipping `Payload` parsing for packets that do not pass the filter.
    /// As a result, only packets containing relevant `Block` data are fully processed.
    ///
    /// # Arguments
    /// * `filter` - A closure that takes a slice of `Block` instances and returns `true` if the packet should be processed,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageIteratorFilteredByBlocks<'_, S, F, B, P, Inner>` - An iterator yielding packets that pass the `Block` filter.
    pub fn filtered_by_blocks<BR: BlockReferredDef<B>, F: FnMut(&[BR]) -> bool>(
        &mut self,
        filter: F,
    ) -> StorageIteratorFilteredByBlocks<'_, S, F, B, BR, P, Inner> {
        StorageIteratorFilteredByBlocks::new(&mut self.inner, &self.slots, filter)
    }

    /// Returns an iterator over `Packet` instances in storage with filtering based on `Packet` content.
    ///
    /// Unlike `Block`-based filtering, this method applies the filter to fully parsed `Packet` instances.
    /// This allows for more detailed inspection of the packet's contents, such as searching within its `Payload`.
    ///
    /// # Arguments
    /// * `filter` - A closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageIteratorFilteredByPacket<'_, S, F, B, P, Inner>` - An iterator yielding packets that satisfy the `Packet` filter.
    pub fn filtered_by_packet<F: FnMut(&PacketDef<B, P, Inner>) -> bool>(
        &mut self,
        filter: F,
    ) -> StorageIteratorFilteredByPacket<'_, S, F, B, P, Inner> {
        StorageIteratorFilteredByPacket::new(&mut self.inner, &self.slots, filter)
    }

    /// Returns an iterator over `Packet` instances in storage with combined filtering by `Block` and `Packet`.
    ///
    /// This method applies two levels of filtering:
    /// - **Block-level filtering (`pfilter`)**: Determines whether a packet should be processed based on its `Block` data.
    ///   If the filter returns `false`, `Payload` parsing is skipped, improving performance.
    /// - **Packet-level filtering (`filter`)**: Applied after the `Packet` is fully parsed, allowing for additional checks
    ///   such as content inspection within `Payload`.
    ///
    /// # Arguments
    /// * `pfilter` - A closure that takes a slice of `Block` instances and returns `true` if the packet should be processed,
    ///   or `false` if it should be skipped.
    /// * `filter` - A closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageIteratorFiltered<'_, S, PF, F, B, P, Inner>` - An iterator yielding packets that pass both filters.
    pub fn filtered<
        BR: BlockReferredDef<B>,
        PF: FnMut(&[BR]) -> bool,
        F: FnMut(&PacketDef<B, P, Inner>) -> bool,
    >(
        &mut self,
        pfilter: PF,
        filter: F,
    ) -> StorageIteratorFiltered<'_, S, PF, F, B, BR, P, Inner> {
        StorageIteratorFiltered::new(&mut self.inner, &self.slots, pfilter, filter)
    }

    /// Attempts to retrieve a `Packet` from storage by its sequential index.
    ///
    /// This method provides direct access to a packet based on its position in the storage, allowing efficient
    /// random access to stored packets.
    ///
    /// # Arguments
    /// * `nth` - The zero-based index of the packet in storage.
    ///
    /// # Returns
    /// * `Ok(Some(PacketDef<B, P, Inner>))` - The packet at the specified index, if found.
    /// * `Ok(None)` - No packet exists at the given index.
    /// * `Err(Error)` - An error occurred while accessing the storage.
    pub fn nth(&mut self, nth: usize) -> Result<Option<PacketDef<B, P, Inner>>, Error> {
        let slot_index = nth / DEFAULT_SLOT_CAPACITY;
        let index_in_slot = nth % DEFAULT_SLOT_CAPACITY;
        let Some(slot) = self.slots.get(slot_index) else {
            return Ok(None);
        };
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

    /// Returns an iterator over `Packet` instances in storage for a specified range of indices.
    ///
    /// This method allows iterating over packets within a given index range, enabling efficient sequential access
    /// to a subset of stored packets.
    ///
    /// # Arguments
    /// * `range` - An inclusive range of packet indices to iterate over.
    ///
    /// # Returns
    /// * `StorageRangeIterator<'_, S, B, P, Inner>` - An iterator yielding packets within the specified range.
    pub fn range(
        &mut self,
        range: RangeInclusive<usize>,
    ) -> StorageRangeIterator<'_, S, B, P, Inner> {
        StorageRangeIterator::new(self, range)
    }

    /// Returns an iterator over `Packet` instances in storage for a specified range with filtering based on `Block` data.
    ///
    /// This filtering method improves performance by skipping `Payload` parsing for packets that do not pass the filter.
    /// As a result, only packets containing relevant `Block` data are fully processed.
    ///
    /// # Arguments
    /// * `range` - An inclusive range of packet indices to iterate over.
    /// * `filter` - A closure that takes a slice of `Block` instances and returns `true` if the packet should be processed,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageRangeIteratorFilteredByBlocks<'_, S, F, B, P, Inner>` - An iterator yielding packets that pass the `Block` filter within the specified range.
    pub fn range_filtered_by_blocks<BR: BlockReferredDef<B>, F: FnMut(&[BR]) -> bool>(
        &mut self,
        range: RangeInclusive<usize>,
        filter: F,
    ) -> StorageRangeIteratorFilteredByBlocks<'_, S, F, B, BR, P, Inner> {
        StorageRangeIteratorFilteredByBlocks::new(self, range, filter)
    }

    /// Returns an iterator over `Packet` instances in storage for a specified range with filtering based on `Packet` content.
    ///
    /// Unlike `Block`-based filtering, this method applies the filter to fully parsed `Packet` instances.
    /// This allows for more detailed inspection of the packet’s contents, such as searching within its `Payload`.
    ///
    /// # Arguments
    /// * `range` - An inclusive range of packet indices to iterate over.
    /// * `filter` - A closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageRangeIteratorFilteredByPacket<'_, S, F, B, P, Inner>` - An iterator yielding packets that satisfy the `Packet` filter within the specified range.
    pub fn range_filtered_by_packet<F: FnMut(&PacketDef<B, P, Inner>) -> bool>(
        &mut self,
        range: RangeInclusive<usize>,
        filter: F,
    ) -> StorageRangeIteratorFilteredByPacket<'_, S, F, B, P, Inner> {
        StorageRangeIteratorFilteredByPacket::new(self, range, filter)
    }

    /// Returns an iterator over `Packet` instances in storage for a specified range with combined filtering by `Block` and `Packet`.
    ///
    /// This method applies two levels of filtering:
    /// - **Block-level filtering (`pfilter`)**: Determines whether a packet should be processed based on its `Block` data.
    ///   If the filter returns `false`, `Payload` parsing is skipped, improving performance.
    /// - **Packet-level filtering (`filter`)**: Applied after the `Packet` is fully parsed, allowing for additional checks
    ///   such as content inspection within `Payload`.
    ///
    /// # Arguments
    /// * `range` - An inclusive range of packet indices to iterate over.
    /// * `pfilter` - A closure that takes a slice of `Block` instances and returns `true` if the packet should be processed,
    ///   or `false` if it should be skipped.
    /// * `filter` - A closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `StorageRangeIteratorFiltered<'_, S, PF, F, B, P, Inner>` - An iterator yielding packets that pass both filters within the specified range.
    pub fn range_filtered<
        BR: BlockReferredDef<B>,
        PF: FnMut(&[BR]) -> bool,
        F: FnMut(&PacketDef<B, P, Inner>) -> bool,
    >(
        &mut self,
        range: RangeInclusive<usize>,
        pfilter: PF,
        filter: F,
    ) -> StorageRangeIteratorFiltered<'_, S, PF, F, B, BR, P, Inner> {
        StorageRangeIteratorFiltered::new(self, range, pfilter, filter)
    }

    /// Attempts to retrieve the `nth` packet from storage that satisfies the `Block` filter.
    ///
    /// This method searches for a packet at the specified index while applying a `Block`-level filter.
    /// If a packet does not pass the filter, it is skipped, and the search continues until the `nth` matching packet is found.
    ///
    /// # Arguments
    /// * `nth` - The zero-based index of the filtered packet in the sequence of matching packets.
    /// * `filter` - A mutable closure that takes a slice of `Block` instances and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `Ok(NthFilteredPacket<B, P, Inner>)` - The `nth` packet that passed the filter.
    /// * `Err(Error)` - An error occurred during retrieval.
    pub(crate) fn nth_filtered_by_blocks<BR: BlockReferredDef<B>, F: FnMut(&[BR]) -> bool>(
        &mut self,
        nth: usize,
        filter: &mut F,
    ) -> Result<NthFilteredPacket<B, P, Inner>, Error> {
        let Some(_) = self.nth_filtered_init(nth)? else {
            return Ok(None);
        };
        match PacketDef::<B, P, Inner>::filtered::<S, BR, _>(&mut self.inner, filter)? {
            LookInStatus::Accepted(size, pkg) => Ok(Some(LookInStatus::Accepted(size, pkg))),
            LookInStatus::Denied(size) => Ok(Some(LookInStatus::Denied(size))),
            LookInStatus::NotEnoughData(needed) => Err(Error::NotEnoughData(needed)),
        }
    }

    /// Attempts to retrieve the `nth` packet from storage that satisfies the `Packet` filter.
    ///
    /// This method searches for a packet at the specified index while applying a `Packet`-level filter.
    /// Unlike `Block` filtering, this filter is applied after the `Packet` is fully parsed, allowing for deeper inspection
    /// of its contents. If a packet does not pass the filter, it is skipped, and the search continues until the `nth` matching packet is found.
    ///
    /// # Arguments
    /// * `nth` - The zero-based index of the filtered packet in the sequence of matching packets.
    /// * `filter` - A mutable closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `Ok(NthFilteredPacket<B, P, Inner>)` - The `nth` packet that passed the filter.
    /// * `Err(Error)` - An error occurred during retrieval.
    pub(crate) fn nth_filtered_by_packet<F: FnMut(&PacketDef<B, P, Inner>) -> bool>(
        &mut self,
        nth: usize,
        filter: &mut F,
    ) -> Result<NthFilteredPacket<B, P, Inner>, Error> {
        let Some(offset) = self.nth_filtered_init(nth)? else {
            return Ok(None);
        };
        let (size, pkg) = match <PacketDef<B, P, Inner> as TryReadFrom>::try_read(&mut self.inner)?
        {
            ReadStatus::Success(pkg) => ((self.inner.stream_position()? - offset) as usize, pkg),
            ReadStatus::NotEnoughData(needed) => {
                return Err(Error::NotEnoughData(needed as usize));
            }
        };
        if !filter(&pkg) {
            Ok(Some(LookInStatus::Denied(size)))
        } else {
            Ok(Some(LookInStatus::Accepted(size, pkg)))
        }
    }

    /// Attempts to retrieve the `nth` packet from storage that satisfies both `Block` and `Packet` filters.
    ///
    /// This method searches for a packet at the specified index while applying two levels of filtering:
    /// - **Block-level filtering (`pfilter`)**: Determines whether a packet should be processed based on its `Block` data.
    ///   If the filter returns `false`, `Payload` parsing is skipped.
    /// - **Packet-level filtering (`filter`)**: Applied after the `Packet` is fully parsed, allowing for deeper inspection
    ///   of its contents.
    ///
    /// If a packet does not pass either filter, it is skipped, and the search continues until the `nth` matching packet is found.
    ///
    /// # Arguments
    /// * `nth` - The zero-based index of the filtered packet in the sequence of matching packets.
    /// * `pfilter` - A mutable closure that takes a slice of `Block` instances and returns `true` if the packet should be processed,
    ///   or `false` if it should be skipped.
    /// * `filter` - A mutable closure that takes a reference to a `PacketDef` instance and returns `true` if the packet should be included,
    ///   or `false` if it should be skipped.
    ///
    /// # Returns
    /// * `Ok(NthFilteredPacket<B, P, Inner>)` - The `nth` packet that passed both filters.
    /// * `Err(Error)` - An error occurred during retrieval.
    pub(crate) fn nth_filtered<
        BR: BlockReferredDef<B>,
        PF: FnMut(&[BR]) -> bool,
        F: FnMut(&PacketDef<B, P, Inner>) -> bool,
    >(
        &mut self,
        nth: usize,
        pfilter: &mut PF,
        filter: &mut F,
    ) -> Result<NthFilteredPacket<B, P, Inner>, Error> {
        let Some(_) = self.nth_filtered_init(nth)? else {
            return Ok(None);
        };
        let (size, pkg) =
            match PacketDef::<B, P, Inner>::filtered::<S, BR, _>(&mut self.inner, pfilter)? {
                LookInStatus::Accepted(size, pkg) => (size, pkg),
                LookInStatus::Denied(size) => return Ok(Some(LookInStatus::Denied(size))),
                LookInStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(needed));
                }
            };
        if !filter(&pkg) {
            Ok(Some(LookInStatus::Denied(size)))
        } else {
            Ok(Some(LookInStatus::Accepted(size, pkg)))
        }
    }

    /// Initializes the retrieval process for the `nth` packet that meets the filtering criteria.
    ///
    /// This method performs an initial lookup to determine the storage position of the `nth` filtered packet.
    /// The result can be used for further processing or direct access to the packet’s location.
    ///
    /// # Arguments
    /// * `nth` - The zero-based index of the filtered packet in the sequence of matching packets.
    ///
    /// # Returns
    /// * `Ok(Some(u64))` - The byte offset of the `nth` packet in storage, if found.
    /// * `Ok(None)` - No matching packet was found at the given index.
    /// * `Err(Error)` - An error occurred while searching for the packet.
    fn nth_filtered_init(&mut self, nth: usize) -> Result<Option<u64>, Error> {
        let slot_index = nth / DEFAULT_SLOT_CAPACITY;
        let index_in_slot = nth % DEFAULT_SLOT_CAPACITY;
        let Some(slot) = self.slots.get(slot_index) else {
            return Ok(None);
        };
        let Some(mut offset) = slot.get_slot_offset(index_in_slot) else {
            return Ok(None);
        };
        offset += self.slots[..slot_index]
            .iter()
            .map(|slot| slot.width() + slot.size())
            .sum::<u64>();
        self.inner.seek(std::io::SeekFrom::Start(offset))?;
        Ok(Some(offset))
    }
}
