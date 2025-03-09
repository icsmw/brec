use crate::*;

#[derive(Default)]
pub struct FreeSlotLocator {
    next: usize,
    slot_offset: u64,
}

impl FreeSlotLocator {
    pub fn next(&mut self, slots: &[Slot]) -> Option<u64> {
        let slot = slots.get(self.next)?;
        let offset = match slot.get_free_slot_offset() {
            Some(offset) => offset,
            None => {
                self.slot_offset += slot.size() + slot.width();
                self.next += 1;
                let slot = slots.get(self.next)?;
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
