mod header;
mod read;
mod write;

use std::ops::RangeInclusive;

use crate::*;
pub(crate) use header::*;

pub static DEFAULT_SLOT_CAPACITY: usize = 100;
pub static STORAGE_SLOT_SIG: [u8; 8] = [166u8, 177u8, 188u8, 199u8, 199u8, 188u8, 177u8, 166u8];

#[derive(Debug)]
pub struct Slot {
    pub lenghts: Vec<u64>,
    pub capacity: u64,
    pub crc: [u8; 4],
}

impl Slot {
    pub fn new(lenghts: Vec<u64>, capacity: u64, crc: [u8; 4]) -> Self {
        Self {
            lenghts,
            capacity,
            crc,
        }
    }
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
    pub fn iter(&self) -> SlotIterator {
        SlotIterator::new(self)
    }
    pub fn get_slot_offset(&self, nth: usize) -> Option<u64> {
        if nth >= self.lenghts.len() {
            return None;
        }
        Some(self.lenghts[..nth].iter().sum::<u64>() + self.size())
    }

    pub fn get_free_slot_offset(&self) -> Option<u64> {
        if let Some(ln) = self.lenghts.last() {
            if ln > &0 {
                return None;
            }
        }
        let free_pos = self.lenghts.iter().position(|ln| ln == &0)?;
        Some(self.lenghts[..free_pos].iter().sum::<u64>() + self.size())
    }
    pub fn overwrite_crc(&mut self) {
        self.crc = self.crc();
    }
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
    fn size(&self) -> u64 {
        SlotHeader::ssize()
            + self.capacity * std::mem::size_of::<u64>() as u64
            + std::mem::size_of::<u32>() as u64
    }
}

impl CrcU32 for Slot {
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
