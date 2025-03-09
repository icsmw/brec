use std::ops::RangeInclusive;

use crate::*;

pub struct PacketsLocatorIterator<'a> {
    next: usize,
    offset: u64,
    last: Option<u64>,
    slots: Vec<SlotIterator<'a>>,
}

impl<'a> PacketsLocatorIterator<'a> {
    pub fn new(slots: &'a [Slot]) -> Self {
        Self {
            next: 0,
            offset: 0,
            last: None,
            slots: slots.iter().map(|slot| slot.iter()).collect(),
        }
    }
}

impl Iterator for PacketsLocatorIterator<'_> {
    type Item = RangeInclusive<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slots.get_mut(self.next)?;
        let location = match slot.next() {
            Some(location) => location,
            None => {
                if let Some(offset) = self.last.take() {
                    self.offset += offset;
                }
                self.next += 1;
                let slot = self.slots.get_mut(self.next)?;
                slot.next()?
            }
        };
        self.last = Some(*location.end());
        Some(RangeInclusive::new(
            self.offset + *location.start(),
            self.offset + *location.end(),
        ))
    }
}

pub struct StorageIterator<
    'a,
    S: std::io::Read + std::io::Seek,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    locator: PacketsLocatorIterator<'a>,
    source: &'a mut S,
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
    pub fn new(source: &'a mut S, slots: &'a [Slot]) -> Self {
        Self {
            locator: PacketsLocatorIterator::new(slots),
            source,
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

    fn next(&mut self) -> Option<Self::Item> {
        let location = self.locator.next()?;
        if let Err(err) = self
            .source
            .seek(std::io::SeekFrom::Start(*location.start()))
        {
            return Some(Err(err.into()));
        }
        match <PacketDef<B, P, Inner> as TryReadFrom>::try_read(&mut self.source) {
            Err(err) => Some(Err(err)),
            Ok(ReadStatus::Success(pkg)) => Some(Ok(pkg)),
            Ok(ReadStatus::NotEnoughData(needed)) => {
                Some(Err(Error::NotEnoughData(needed as usize)))
            }
        }
    }
}

pub struct StorageIteratorFiltered<
    'a,
    S: std::io::Read + std::io::Seek,
    F: FnMut(&[B]) -> bool,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    locator: PacketsLocatorIterator<'a>,
    source: &'a mut S,
    predicate: F,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        'a,
        S: std::io::Read + std::io::Seek,
        F: FnMut(&[B]) -> bool,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageIteratorFiltered<'a, S, F, B, P, Inner>
{
    pub fn new(source: &'a mut S, slots: &'a [Slot], predicate: F) -> Self {
        Self {
            locator: PacketsLocatorIterator::new(slots),
            source,
            predicate,
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
        }
    }
}

impl<
        S: std::io::Read + std::io::Seek,
        F: FnMut(&[B]) -> bool,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageIteratorFiltered<'_, S, F, B, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let location = self.locator.next()?;
            if let Err(err) = self
                .source
                .seek(std::io::SeekFrom::Start(*location.start()))
            {
                return Some(Err(err.into()));
            }
            match PacketDef::<B, P, Inner>::filtered(self.source, &mut self.predicate) {
                Err(err) => return Some(Err(err)),
                Ok(LookInStatus::Accepted(_, pkg)) => return Some(Ok(pkg)),
                Ok(LookInStatus::Denied(_)) => continue,
                Ok(LookInStatus::NotEnoughData(needed)) => {
                    return Some(Err(Error::NotEnoughData(needed)))
                }
            }
        }
    }
}

pub struct StorageRangeIterator<
    'a,
    S: std::io::Read + std::io::Write + std::io::Seek,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    storage: &'a mut StorageDef<S, B, P, Inner>,
    end: usize,
    current: usize,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        'a,
        S: std::io::Read + std::io::Write + std::io::Seek,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageRangeIterator<'a, S, B, P, Inner>
{
    pub fn new(storage: &'a mut StorageDef<S, B, P, Inner>, range: RangeInclusive<usize>) -> Self {
        Self {
            storage,
            end: *range.end(),
            current: *range.start(),
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
    > Iterator for StorageRangeIterator<'_, S, B, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let item = self.storage.nth(self.current);
        self.current += 1;
        match item {
            Ok(None) => None,
            Ok(Some(packet)) => Some(Ok(packet)),
            Err(err) => Some(Err(err)),
        }
    }
}

pub struct StorageRangeIteratorFiltered<
    'a,
    S: std::io::Read + std::io::Write + std::io::Seek,
    F: FnMut(&[B]) -> bool,
    B: BlockDef,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    storage: &'a mut StorageDef<S, B, P, Inner>,
    end: usize,
    current: usize,
    predicate: F,
    _block: std::marker::PhantomData<B>,
    _payload: std::marker::PhantomData<P>,
    _payload_inner: std::marker::PhantomData<Inner>,
}

impl<
        'a,
        S: std::io::Read + std::io::Write + std::io::Seek,
        F: FnMut(&[B]) -> bool,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > StorageRangeIteratorFiltered<'a, S, F, B, P, Inner>
{
    pub fn new(
        storage: &'a mut StorageDef<S, B, P, Inner>,
        range: RangeInclusive<usize>,
        predicate: F,
    ) -> Self {
        Self {
            storage,
            end: *range.end(),
            current: *range.start(),
            predicate,
            _block: std::marker::PhantomData,
            _payload: std::marker::PhantomData,
            _payload_inner: std::marker::PhantomData,
        }
    }
}

impl<
        S: std::io::Read + std::io::Write + std::io::Seek,
        F: FnMut(&[B]) -> bool,
        B: BlockDef,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Iterator for StorageRangeIteratorFiltered<'_, S, F, B, P, Inner>
{
    type Item = Result<PacketDef<B, P, Inner>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current >= self.end {
                return None;
            }
            let item = self.storage.nth_filtered(self.current, &mut self.predicate);
            self.current += 1;
            match item {
                Ok(None) => return None,
                Ok(Some(LookInStatus::Accepted(_, packet))) => return Some(Ok(packet)),
                Ok(Some(LookInStatus::Denied(_))) => continue,
                Ok(Some(LookInStatus::NotEnoughData(needed))) => {
                    return Some(Err(Error::NotEnoughData(needed)))
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
