use std::ops::RangeInclusive;

use crate::*;

#[derive(Default)]
pub struct PacketLocator {
    next: usize,
    offset: u64,
    last: Option<u64>,
}

impl PacketLocator {
    pub fn next(&mut self, slots: &mut [Slot]) -> Option<RangeInclusive<u64>> {
        let slot = slots.get_mut(self.next)?;
        let location = match slot.next_packet_position() {
            Some(location) => location,
            None => {
                if let Some(offset) = self.last.take() {
                    self.offset += offset;
                }
                self.next += 1;
                let slot = slots.get_mut(self.next)?;
                slot.next_packet_position()?
            }
        };
        self.last = Some(*location.end());
        Some(RangeInclusive::new(
            self.offset + *location.start(),
            self.offset + *location.end(),
        ))
    }
    pub fn drop(&mut self) {
        self.offset = 0;
        self.next = 0;
    }
}

#[derive(Default)]
pub struct FreeSlotLocator {
    next: usize,
    slot_offset: u64,
}

impl FreeSlotLocator {
    pub fn next(&mut self, slots: &mut [Slot]) -> Option<u64> {
        let slot = slots.get_mut(self.next)?;
        let offset = match slot.get_free_slot_offset() {
            Some(offset) => offset,
            None => {
                self.slot_offset += slot.size() + slot.width();
                self.next += 1;
                let slot = slots.get_mut(self.next)?;
                slot.get_free_slot_offset()?
            }
        };
        Some(self.slot_offset + offset)
    }
    pub fn insert(&mut self, slots: &mut [Slot], length: u64) -> Result<(), Error> {
        let slot = slots
            .get_mut(self.next)
            .ok_or(Error::CannotInsertIntoSlot)?;
        slot.insert(length)
    }
    pub fn current(&self) -> (usize, u64) {
        (self.next, self.slot_offset)
    }
}
