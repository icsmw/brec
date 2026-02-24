use crate::traits::Size;
use crate::*;

pub struct AnchoredSlot {
    pub inner: Slot,
    pub offset: u64,
}

impl AnchoredSlot {
    pub fn new(inner: Slot, offset: u64) -> Self {
        Self { inner, offset }
    }

    pub fn width(&self) -> u64 {
        self.inner.width()
    }

    pub fn iter(&self) -> SlotIterator<'_> {
        SlotIterator::new(&self.inner)
    }

    pub fn get_slot_offset(&self, nth: usize) -> Option<u64> {
        self.inner.get_slot_offset(nth)
    }

    pub fn get_free_slot_index(&self) -> Option<usize> {
        self.inner.get_free_slot_index()
    }

    pub fn get_free_slot_offset(&self) -> Option<u64> {
        self.inner.get_free_slot_offset()
    }

    pub fn insert(&mut self, length: u64) -> Result<(), Error> {
        self.inner.insert(length)
    }

    pub fn is_empty(&self, nth: usize) -> Result<bool, Error> {
        self.inner.is_empty(nth)
    }

    pub fn overwrite_crc(&mut self) {
        self.inner.overwrite_crc()
    }
}

impl Size for AnchoredSlot {
    /// Computes the full size of the slot in bytes, including:
    /// - slot header
    /// - all capacity entries (`u64`)
    /// - CRC field
    fn size(&self) -> u64 {
        self.inner.size()
    }
}

impl CrcU32 for AnchoredSlot {
    /// Computes a CRC over the capacity and length fields.
    fn crc(&self) -> [u8; 4] {
        self.inner.crc()
    }
}
