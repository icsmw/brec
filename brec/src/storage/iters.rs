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
    type Item = Result<ReadStatus<PacketDef<B, P, Inner>>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let location = self.locator.next()?;
        if let Err(err) = self
            .source
            .seek(std::io::SeekFrom::Start(*location.start()))
        {
            return Some(Err(err.into()));
        }
        Some(<PacketDef<B, P, Inner> as TryReadFrom>::try_read(
            &mut self.source,
        ))
    }
}
