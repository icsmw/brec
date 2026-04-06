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
    /// * `slots` - A slice of `Slot` structures to scan.
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
    /// * `slots` - A mutable slice of `Slot` values.
    /// * `length` - The number of bytes to insert.
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

    pub fn setup<'a, I: Iterator<Item = &'a Slot>>(&mut self, slots: I) {
        self.next = 0;
        self.slot_offset = 0;
        for slot in slots {
            if slot.get_free_slot_offset().is_some() {
                break;
            }
            self.slot_offset += slot.size() + slot.width();
            self.next += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FreeSlotLocator;
    use crate::{Error, Size, Slot};

    #[test]
    fn locator_next_current_and_insert_on_first_slot() {
        let mut slots = vec![Slot::new(vec![0, 0], 2, [0; 4])];
        slots[0].overwrite_crc();
        let mut locator = FreeSlotLocator::default();

        let next = locator.next(&slots).expect("free offset expected");
        assert_eq!(next, slots[0].size());
        assert_eq!(locator.current(), (0, 0));

        locator.insert(&mut slots, 15).expect("insert into first slot");
        assert_eq!(slots[0].lenghts[0], 15);
    }

    #[test]
    fn locator_next_skips_full_slots() {
        let mut first = Slot::new(vec![5, 7], 2, [0; 4]);
        first.overwrite_crc();

        let mut second = Slot::new(vec![0, 0], 2, [0; 4]);
        second.overwrite_crc();

        let slots = vec![first, second];
        let mut locator = FreeSlotLocator::default();

        let next = locator.next(&slots).expect("must find free offset in second slot");
        let expected_slot_offset = slots[0].size() + slots[0].width();
        let expected_next = expected_slot_offset + slots[1].size();

        assert_eq!(next, expected_next);
        assert_eq!(locator.current(), (1, expected_slot_offset));
    }

    #[test]
    fn locator_next_and_insert_fail_when_out_of_bounds_or_full() {
        let mut full = Slot::new(vec![1], 1, [0; 4]);
        full.overwrite_crc();
        let slots = vec![full];

        let mut locator = FreeSlotLocator::default();
        assert_eq!(locator.next(&slots), None);
        assert!(matches!(
            locator.insert(&mut [] as &mut [Slot], 1),
            Err(Error::CannotInsertIntoSlot)
        ));
    }

    #[test]
    fn locator_setup_positions_on_first_non_full_slot() {
        let mut s0 = Slot::new(vec![10], 1, [0; 4]);
        s0.overwrite_crc();
        let mut s1 = Slot::new(vec![20], 1, [0; 4]);
        s1.overwrite_crc();
        let mut s2 = Slot::new(vec![0, 0], 2, [0; 4]);
        s2.overwrite_crc();
        let slots = vec![s0, s1, s2];

        let mut locator = FreeSlotLocator::default();
        locator.setup(slots.iter());

        let expected_offset = slots[0].size() + slots[0].width() + slots[1].size() + slots[1].width();
        assert_eq!(locator.current(), (2, expected_offset));

        let next = locator.next(&slots).expect("next free offset in third slot");
        assert_eq!(next, expected_offset + slots[2].size());
    }
}
