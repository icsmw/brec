use std::{
    io::{BufRead, Cursor},
    ops::RangeInclusive,
};

use crate::*;

/// Iterator over a sequence of `Slot`s, yielding the byte ranges that contain actual packet data.
///
/// Each iteration returns a `RangeInclusive<u64>` representing the location of the used data region
/// within the slot. The offset includes the size of the slot header and accounts for cumulative offset.
///
/// The iterator skips over empty slots and automatically adjusts for the internal layout.
///
/// Useful for scanning files or buffers that store serialized packets in slot-based format.
pub struct PacketsLocatorIterator<'a> {
    next: usize,
    offset: u64,
    slots: &'a [Slot],
}

impl<'a> PacketsLocatorIterator<'a> {
    /// Creates a new `PacketsLocatorIterator` over the provided slice of slots.
    pub fn new(slots: &'a [Slot]) -> Self {
        Self {
            next: 0,
            offset: 0,
            slots,
        }
    }
}

impl Iterator for PacketsLocatorIterator<'_> {
    type Item = RangeInclusive<u64>;

    /// Returns the next occupied range of packet data, or `None` if finished.
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slots.get(self.next)?;
        let slot_width = slot.width();
        if slot_width == 0 {
            return None;
        }
        let location = RangeInclusive::new(
            self.offset + slot.size(),
            self.offset + slot_width + slot.size(),
        );
        self.offset += slot_width + slot.size();
        self.next += 1;
        Some(location)
    }
}

/// An iterator over stored packets distributed across multiple slots.
///
/// `StorageIterator` reads packets from a `Read + Seek` source (e.g. file or memory stream)
/// using a list of `Slot`s and their recorded layout. It internally tracks the position and
/// reuses an internal buffer (`Cursor<Vec<u8>>`) to efficiently read packet data.
///
/// It yields deserialized `PacketDef<B, P, Inner>` values or parsing errors.
///
/// # Type Parameters
/// - `S`: Source implementing `Read + Seek`
/// - `B`: Block type (must implement `BlockDef`)
/// - `P`: Payload container type (must implement `PayloadDef`)
/// - `Inner`: Inner payload object (must implement `PayloadInnerDef`)
pub struct StorageIterator<
    'a,
    S: std::io::Read + std::io::Seek,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    locator: PacketsLocatorIterator<'a>,
    source: &'a mut S,
    buffer: Cursor<Vec<u8>>,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        'a,
        S: std::io::Read + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageIterator<'a, S, B, P, Inner>
{
    /// Constructs a new `StorageIterator` from the given stream and slot layout.
    pub fn new(source: &'a mut S, slots: &'a [Slot]) -> Self {
        Self {
            locator: PacketsLocatorIterator::new(slots),
            source,
            buffer: Cursor::new(Vec::new()),
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
        }
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageIterator<'_, S, B, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    /// Reads and yields the next packet located in the slots.
    ///
    /// Loads the slot's region into an internal buffer and calls `PacketDef::read`.
    /// Returns `None` if all slots are exhausted.
    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.fill_buf().unwrap().is_empty() {
            let location = self.locator.next()?;
            if let Err(err) = self
                .source
                .seek(std::io::SeekFrom::Start(*location.start()))
            {
                return Some(Err(err.into()));
            }
            let size = (location.end() - location.start()) as usize;
            let mut inner = vec![0u8; size];
            self.source.read_exact(&mut inner).unwrap();
            self.buffer = Cursor::new(inner);
        }
        match <PacketDef<B, P, Inner> as ReadFrom>::read(&mut self.buffer) {
            Err(err) => Some(Err(err)),
            Ok(pkg) => Some(Ok(pkg)),
        }
    }
}

/// An iterator over packets stored in slots with rule-based filtering.
///
/// This iterator functions like `StorageIterator`, but applies `RulesDef`-based
/// filters to decide whether to yield, skip, or reject packets. The filtering is performed
/// during parsing using `PacketDef::filtered`, which allows:
/// - filtering by blocks (`FilterByBlocks`)
/// - filtering by payload (`FilterByPayload`)
/// - filtering by fully parsed packet (`Filter`)
///
/// # Type Parameters
/// - `S`: Input stream (`Read + Seek`)
/// - `B`: Block type
/// - `BR`: Referred block type for rule filtering
/// - `P`: Payload wrapper type
/// - `Inner`: Inner payload type
pub struct StorageFilteredIterator<
    'a,
    S: std::io::Read + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    locator: PacketsLocatorIterator<'a>,
    source: &'a mut S,
    rules: &'a RulesDef<B, BR, P, Inner>,
    buffer: Cursor<Vec<u8>>,
}

impl<
        'a,
        S: std::io::Read + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageFilteredIterator<'a, S, B, BR, P, Inner>
{
    /// Constructs a new filtered packet iterator from the given stream, slots and rules.
    pub fn new(source: &'a mut S, slots: &'a [Slot], rules: &'a RulesDef<B, BR, P, Inner>) -> Self {
        Self {
            locator: PacketsLocatorIterator::new(slots),
            source,
            rules,
            buffer: Cursor::new(Vec::new()),
        }
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageFilteredIterator<'_, S, B, BR, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    /// Attempts to read and yield the next packet that passes all configured rules.
    ///
    /// Internally uses `PacketDef::filtered` to apply:
    /// - block-level filtering
    /// - payload-level filtering
    /// - full-packet filtering
    ///
    /// Returns `Some(Ok(...))` if accepted, skips if denied, and `Err(...)` on error.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.buffer.fill_buf().unwrap().is_empty() {
                let location = self.locator.next()?;
                if let Err(err) = self
                    .source
                    .seek(std::io::SeekFrom::Start(*location.start()))
                {
                    return Some(Err(err.into()));
                }
                let size = (location.end() - location.start()) as usize;
                let mut inner = vec![0u8; size];
                self.source.read_exact(&mut inner).unwrap();
                self.buffer = Cursor::new(inner);
            }
            match PacketDef::filtered(&mut self.buffer, self.rules) {
                Ok(LookInStatus::Accepted(_, packet)) => return Some(Ok(packet)),
                Ok(LookInStatus::Denied(_)) => {
                    continue;
                }
                Ok(LookInStatus::NotEnoughData(needed)) => {
                    return Some(Err(Error::NotEnoughData(needed)))
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

/// An iterator over a specified range of packets within a `StorageDef`.
///
/// Unlike `StorageIterator`, this variant yields a bounded number of packets starting from a specific index.
/// It uses the internal `storage.nth(n)` method to fetch each packet by logical index.
///
/// # Type Parameters
/// - `S`: Underlying storage stream (`Read + Write + Seek`)
/// - `B`: Block type
/// - `BR`: Referred block type
/// - `P`: Payload type wrapper
/// - `Inner`: Inner payload object
pub struct StorageRangeIterator<
    'a,
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    storage: &'a mut StorageDef<S, B, BR, P, Inner>,
    len: usize,
    from: usize,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        'a,
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageRangeIterator<'a, S, B, BR, P, Inner>
{
    /// Creates a new range-based iterator from the given `storage`, starting at `from`, returning up to `len` items.
    pub fn new(storage: &'a mut StorageDef<S, B, BR, P, Inner>, from: usize, len: usize) -> Self {
        Self {
            storage,
            len,
            from,
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
        }
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageRangeIterator<'_, S, B, BR, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    /// Returns the next packet from the range by calling `storage.nth(current_index)`.
    ///
    /// Ends after yielding `len` elements or if an error occurs.
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let item = self.storage.nth(self.from);
        self.from += 1;
        self.len -= 1;
        match item {
            Ok(None) => None,
            Ok(Some(packet)) => Some(Ok(packet)),
            Err(err) => Some(Err(err)),
        }
    }
}

/// An iterator over a specific range of packets from storage with rule-based filtering.
///
/// Similar to `StorageRangeIterator`, but each packet is passed through the configured filtering rules.
/// Internally uses `storage.nth_filtered(index)` to fetch and filter packets.
///
/// Only packets that match all rule conditions are yielded (`LookInStatus::Accepted`).
///
/// # Type Parameters
/// - `S`: Source stream with `Read + Write + Seek`
/// - `B`: Block type
/// - `BR`: Block reference type for filtering
/// - `P`: Payload container type
/// - `Inner`: Inner payload object
pub struct StorageRangeFilteredIterator<
    'a,
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    storage: &'a mut StorageDef<S, B, BR, P, Inner>,
    len: usize,
    from: usize,
}

impl<
        'a,
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageRangeFilteredIterator<'a, S, B, BR, P, Inner>
{
    /// Creates a new filtered range-based iterator from `storage`, starting at `from`, returning up to `len` matching packets.
    pub fn new(storage: &'a mut StorageDef<S, B, BR, P, Inner>, from: usize, len: usize) -> Self {
        Self { storage, len, from }
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageRangeFilteredIterator<'_, S, B, BR, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    /// Attempts to read and yield the next packet in range that passes all filtering rules.
    ///
    /// Skips over packets that are denied, stops when `len` is exhausted or `None` is returned from storage.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.len == 0 {
                return None;
            }
            let item = self.storage.nth_filtered(self.from);
            self.from += 1;
            match item {
                Ok(None) => return None,
                Ok(Some(LookInStatus::Accepted(_, packet))) => {
                    self.len -= 1;
                    return Some(Ok(packet));
                }
                Ok(Some(LookInStatus::Denied(_))) => {
                    continue;
                }
                Ok(Some(LookInStatus::NotEnoughData(needed))) => {
                    return Some(Err(Error::NotEnoughData(needed)));
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
