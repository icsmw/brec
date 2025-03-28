use crate::*;

/// A helper structure for locating and reserving free space in a sequence of `Slot` entries.
///
/// `FreeSlotLocator` is used to find the next available offset for writing data
/// and to manage the process of inserting data into a slot-based structure (e.g. file or buffer).
///
/// The locator keeps track of:
/// - the index of the current slot (`next`)
/// - the byte offset to the start of that slot (`slot_offset`)
///
/// This allows efficiently iterating through `Slot`s while preserving position state.
#[derive(Default)]
pub struct FreeSlotLocator {
    next: usize,
    slot_offset: u64,
}

impl FreeSlotLocator {
    /// Returns the next available free offset across the slots.
    ///
    /// Internally advances to the next slot if the current one is full.
    ///
    /// # Arguments
    /// * `slots` – A slice of `Slot` structures to scan.
    ///
    /// # Returns
    /// The absolute offset in the full space, or `None` if no free slot was found.
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

    /// Attempts to insert data of the given `length` into the current slot.
    ///
    /// # Arguments
    /// * `slots` – A mutable slice of `Slot` values.
    /// * `length` – The number of bytes to insert.
    ///
    /// # Errors
    /// Returns `Error::CannotInsertIntoSlot` if the current slot is not available or full.
    pub fn insert(&mut self, slots: &mut [Slot], length: u64) -> Result<(), Error> {
        let slot = slots
            .get_mut(self.next)
            .ok_or(Error::CannotInsertIntoSlot)?;
        slot.insert(length)
    }

    /// Returns the current slot index and its starting absolute offset.
    pub fn current(&self) -> (usize, u64) {
        (self.next, self.slot_offset)
    }

    pub fn setup(&mut self, slots: &[Slot]) {
        for slot in slots.iter() {
            if slot.get_free_slot_offset().is_some() {
                break;
            }
            self.slot_offset += slot.size() + slot.width();
            self.next += 1;
        }
    }
}
