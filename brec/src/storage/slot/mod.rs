mod header;
mod read;
mod write;

use std::ops::RangeInclusive;

use crate::*;
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

    /// Returns the total used width (sum of all non-zero chunk lengths).
    pub fn width(&self) -> u64 {
        if let Some(ln) = self.lenghts.last() {
            if ln > &0 {
                return self.lenghts.iter().sum();
            }
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

    /// Returns the offset to the first available free slot, or `None` if the slot is full.
    pub fn get_free_slot_offset(&self) -> Option<u64> {
        if let Some(ln) = self.lenghts.last() {
            if ln > &0 {
                return None;
            }
        }
        let free_pos = self.lenghts.iter().position(|ln| ln == &0)?;
        Some(self.lenghts[..free_pos].iter().sum::<u64>() + self.size())
    }

    /// Returns the indoex of the first available free slot, or `None` if the slot is full.
    pub fn get_free_slot_index(&self) -> Option<usize> {
        if let Some(ln) = self.lenghts.last() {
            if ln > &0 {
                return None;
            }
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
