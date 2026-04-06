use crate::traits::Size;
use crate::*;

/// An anchored slot is a wrapper around a `Slot` that includes an offset indicating its position in storage.
pub struct AnchoredSlot {
    /// The inner `Slot` containing the chunk lengths, capacity, and CRC.
    pub inner: Slot,
    /// The byte offset in storage where this slot is located.
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

#[cfg(test)]
mod tests {
    use crate::{AnchoredSlot, CrcU32, Size, Slot};

    #[test]
    fn anchored_slot_delegates_to_inner_slot() {
        let mut inner = Slot::new(vec![0, 0], 2, [0; 4]);
        inner.overwrite_crc();
        let mut anchored = AnchoredSlot::new(inner, 42);

        assert_eq!(anchored.offset, 42);
        assert_eq!(anchored.size(), anchored.inner.size());
        assert_eq!(anchored.crc(), anchored.inner.crc());
        assert_eq!(anchored.get_free_slot_index(), Some(0));
        assert_eq!(anchored.get_free_slot_offset(), Some(anchored.inner.size()));

        anchored.insert(11).expect("insert first");
        assert_eq!(anchored.get_free_slot_index(), Some(1));
        assert_eq!(anchored.get_slot_offset(1), Some(anchored.inner.size() + 11));
        assert_eq!(anchored.width(), 11);
        assert!(matches!(anchored.is_empty(0), Ok(false)));

        let ranges: Vec<_> = anchored.iter().collect();
        assert_eq!(ranges.len(), 1);
        assert_eq!(*ranges[0].start(), anchored.inner.size());
        assert_eq!(*ranges[0].end(), anchored.inner.size() + 11);
    }
}
