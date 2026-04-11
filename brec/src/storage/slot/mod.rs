mod anchored;
mod header;
mod read;
mod write;

use std::ops::RangeInclusive;

use crate::*;
pub(crate) use anchored::*;
pub(crate) use header::*;

/// Default slot capacity constant.
pub static DEFAULT_SLOT_CAPACITY: usize = 500;
/// Signature prefix used to identify serialized slot headers.
pub static STORAGE_SLOT_SIG: [u8; 8] = [166u8, 177u8, 188u8, 199u8, 199u8, 188u8, 177u8, 166u8];

/// Fixed-size data structure representing a storage slot containing multiple data regions.
///
/// A `Slot` keeps track of inserted data chunk lengths, a maximum capacity, and a CRC checksum.
/// It supports inserting new data, querying offsets, and iterating over used regions.
///
/// Each slot contains:
/// - a vector of `lenghts` (`Vec<u64>`) where each non-zero value represents a data chunk
/// - a declared `capacity` (maximum number of chunks)
/// - a CRC checksum for validation
///
/// Slot entries are zero-initialized and grow in-place until the capacity is reached.
///
/// The constant signature `STORAGE_SLOT_SIG` is used when persisting this structure to disk.
#[derive(Debug)]
pub struct Slot {
    /// List of chunk lengths (zero = unused).
    pub lenghts: Vec<u64>,

    /// Maximum number of entries this slot can hold.
    pub capacity: u64,

    /// CRC over `capacity` and `lenghts` content.
    pub crc: [u8; 4],
}

impl Slot {
    /// Constructs a new `Slot` with specified lengths, capacity and CRC.
    pub fn new(lenghts: Vec<u64>, capacity: u64, crc: [u8; 4]) -> Self {
        Self {
            lenghts,
            capacity,
            crc,
        }
    }

    /// Expands the slot into its components: free slot offset, free slot index, and CRC.
    pub fn expand(&self) -> (Option<u64>, Option<usize>, [u8; 4]) {
        (
            self.get_free_slot_offset(),
            self.get_free_slot_index(),
            self.crc,
        )
    }

    /// Returns the total used width (sum of all non-zero chunk lengths).
    pub fn width(&self) -> u64 {
        if self.is_full() {
            return self.lenghts.iter().sum();
        }
        let Some(free_pos) = self.lenghts.iter().position(|ln| ln == &0) else {
            return self.lenghts.iter().sum();
        };
        self.lenghts[..free_pos].iter().sum()
    }

    /// Returns an iterator over active ranges in the slot.
    pub fn iter(&self) -> SlotIterator<'_> {
        SlotIterator::new(self)
    }

    /// Computes the offset (in bytes) to the start of the `nth` chunk.
    ///
    /// Returns `None` if index is out of bounds.
    pub fn get_slot_offset(&self, nth: usize) -> Option<u64> {
        if nth >= self.lenghts.len() {
            return None;
        }
        Some(self.lenghts[..nth].iter().sum::<u64>() + self.size())
    }

    /// Returns whether the `nth` chunk is unused (i.e., zero-length).
    ///
    /// # Errors
    /// Returns `Error::OutOfBounds` if index is invalid.
    pub fn is_empty(&self, nth: usize) -> Result<bool, Error> {
        self.lenghts
            .get(nth)
            .map(|l| l == &0)
            .ok_or(Error::OutOfBounds(self.lenghts.len(), nth))
    }

    /// Counts the number of used (non-zero) chunks in the slot.
    pub fn count(&self) -> usize {
        self.lenghts.iter().filter(|&&ln| ln > 0).count()
    }

    /// Checks if the chunk at the given index is used (non-zero length).
    ///
    /// # Arguments
    /// * `idx` - The index of the chunk to check.
    pub fn is_used(&self, idx: usize) -> bool {
        self.lenghts.get(idx).is_some_and(|ln| *ln > 0)
    }

    /// Calculates the byte offset to the start of the chunk at the given index.
    ///
    /// # Arguments
    /// * `idx` - The index of the chunk to calculate the offset for.
    ///
    /// # Returns
    /// * `Some(u64)` - The byte offset to the start of the chunk if the index is valid and the chunk is used.
    /// * `None` - If the index is out of bounds or the chunk at the index is unused (zero-length).
    pub fn offset_of(&self, idx: usize) -> Option<u64> {
        if idx == 0 {
            return Some(0);
        }
        if idx >= self.lenghts.len() || !self.is_used(idx) {
            return None;
        }
        Some(self.lenghts[..idx].iter().sum::<u64>())
    }

    /// Checks if slot has space
    pub fn is_full(&self) -> bool {
        if let Some(ln) = self.lenghts.last()
            && ln > &0
        {
            true
        } else {
            false
        }
    }

    /// Returns the offset to the first available free slot, or `None` if the slot is full.
    pub fn get_free_slot_offset(&self) -> Option<u64> {
        if self.is_full() {
            return None;
        }
        let free_pos = self.lenghts.iter().position(|ln| ln == &0)?;
        Some(self.lenghts[..free_pos].iter().sum::<u64>() + self.size())
    }

    /// Returns the indoex of the first available free slot, or `None` if the slot is full.
    pub fn get_free_slot_index(&self) -> Option<usize> {
        if self.is_full() {
            return None;
        }
        self.lenghts.iter().position(|ln| ln == &0)
    }

    /// Recomputes and stores the CRC based on current contents.
    pub fn overwrite_crc(&mut self) {
        self.crc = self.crc();
    }

    /// Inserts a new chunk into the first available slot.
    ///
    /// Updates the CRC accordingly.
    ///
    /// # Errors
    /// Returns `Error::CannotInsertIntoSlot` if no space is available.
    pub fn insert(&mut self, length: u64) -> Result<(), Error> {
        let free_pos = self
            .lenghts
            .iter()
            .position(|ln| ln == &0)
            .ok_or(Error::CannotInsertIntoSlot)?;
        self.lenghts[free_pos] = length;
        self.overwrite_crc();
        Ok(())
    }
}

impl Default for Slot {
    /// Creates an empty `Slot` with default capacity and zero-initialized lengths.
    fn default() -> Self {
        let mut slot = Self::new(
            vec![0u64; DEFAULT_SLOT_CAPACITY],
            DEFAULT_SLOT_CAPACITY as u64,
            [0u8; 4],
        );
        slot.overwrite_crc();
        slot
    }
}

impl Size for Slot {
    /// Computes the full size of the slot in bytes, including:
    /// - slot header
    /// - all capacity entries (`u64`)
    /// - CRC field
    fn size(&self) -> u64 {
        SlotHeader::ssize()
            + self.capacity * std::mem::size_of::<u64>() as u64
            + std::mem::size_of::<u32>() as u64
    }
}

impl CrcU32 for Slot {
    /// Computes a CRC over the capacity and length fields.
    fn crc(&self) -> [u8; 4] {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.capacity.to_le_bytes());
        hasher.update(
            &self
                .lenghts
                .iter()
                .flat_map(|ln| ln.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
        hasher.finalize().to_le_bytes()
    }
}

pub struct SlotIterator<'a> {
    slot: &'a Slot,
    next: usize,
    offset: u64,
}

impl<'a> SlotIterator<'a> {
    /// Creates a new iterator for the given slot.
    pub fn new(slot: &'a Slot) -> Self {
        SlotIterator {
            slot,
            next: 0,
            offset: 0,
        }
    }
}

impl Iterator for SlotIterator<'_> {
    type Item = RangeInclusive<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        let ln = self.slot.lenghts.get(self.next)?;
        if ln == &0 {
            return None;
        }
        let range = RangeInclusive::new(
            self.offset + self.slot.size(),
            self.offset + *ln + self.slot.size(),
        );
        self.next += 1;
        self.offset += *ln;
        Some(range)
    }
}

#[cfg(test)]
mod tests {
    use super::Slot;
    use crate::{CrcU32, Error, STORAGE_SLOT_SIG, Size, StaticSize};

    #[test]
    fn slot_default_shape_and_crc_are_valid() {
        let slot = Slot::default();
        assert_eq!(slot.lenghts.len(), crate::DEFAULT_SLOT_CAPACITY);
        assert_eq!(slot.capacity as usize, crate::DEFAULT_SLOT_CAPACITY);
        assert_eq!(slot.count(), 0);
        assert!(!slot.is_full());
        assert_eq!(slot.crc, slot.crc());
        assert_eq!(
            slot.size(),
            crate::storage::slot::SlotHeader::ssize()
                + slot.capacity * std::mem::size_of::<u64>() as u64
                + std::mem::size_of::<u32>() as u64
        );
        let _ = STORAGE_SLOT_SIG; // ensure symbol is referenced in slot tests
    }

    #[test]
    fn slot_insert_offsets_iter_expand_and_helpers() {
        let mut slot = Slot::new(vec![0, 0, 0], 3, [0; 4]);
        slot.overwrite_crc();

        assert_eq!(slot.get_free_slot_index(), Some(0));
        assert_eq!(slot.get_free_slot_offset(), Some(slot.size()));
        assert_eq!(slot.get_slot_offset(0), Some(slot.size()));
        assert_eq!(slot.offset_of(0), Some(0));
        assert!(!slot.is_used(0));
        assert_eq!(slot.width(), 0);

        slot.insert(10).expect("insert first chunk");
        slot.insert(20).expect("insert second chunk");

        assert_eq!(slot.count(), 2);
        assert!(slot.is_used(0));
        assert!(slot.is_used(1));
        assert!(!slot.is_used(2));
        assert_eq!(slot.offset_of(1), Some(10));
        assert_eq!(slot.get_slot_offset(1), Some(slot.size() + 10));
        assert_eq!(slot.width(), 30);

        let ranges: Vec<_> = slot.iter().collect();
        assert_eq!(ranges.len(), 2);
        assert_eq!(*ranges[0].start(), slot.size());
        assert_eq!(*ranges[0].end(), slot.size() + 10);
        assert_eq!(*ranges[1].start(), slot.size() + 10);
        assert_eq!(*ranges[1].end(), slot.size() + 30);

        let (free_off, free_idx, crc) = slot.expand();
        assert_eq!(free_idx, Some(2));
        assert_eq!(free_off, Some(slot.size() + 30));
        assert_eq!(crc, slot.crc);

        slot.insert(5).expect("insert third chunk");
        assert!(slot.is_full());
        assert_eq!(slot.get_free_slot_index(), None);
        assert_eq!(slot.get_free_slot_offset(), None);
        assert!(matches!(slot.insert(1), Err(Error::CannotInsertIntoSlot)));
    }

    #[test]
    fn slot_empty_and_bounds_checks() {
        let slot = Slot::new(vec![11, 0], 2, [0; 4]);
        assert!(matches!(slot.is_empty(0), Ok(false)));
        assert!(matches!(slot.is_empty(1), Ok(true)));
        assert!(matches!(slot.is_empty(2), Err(Error::OutOfBounds(2, 2))));
        assert_eq!(slot.get_slot_offset(2), None);
        assert_eq!(slot.offset_of(2), None);
    }
}
