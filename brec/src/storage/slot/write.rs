use crate::*;

/// Serializes a `Slot` into a contiguous byte buffer.
///
/// The buffer layout includes:
/// - [8 bytes] signature (`STORAGE_SLOT_SIG`)
/// - [8 bytes] capacity
/// - [N × 8 bytes] lengths (each as `u64`, where N = `capacity`)
/// - [4 bytes] CRC
fn get_buffer(slot: &Slot) -> Vec<u8> {
    let mut buffer = vec![0u8; slot.size() as usize];
    let mut offset = 0;
    buffer[offset..offset + 8usize].copy_from_slice(&STORAGE_SLOT_SIG);
    offset += 8usize;
    buffer[offset..offset + 8usize].copy_from_slice(&slot.capacity.to_le_bytes());
    offset += 8usize;
    for lenght in slot.lenghts.iter() {
        buffer[offset..offset + 8usize].copy_from_slice(&lenght.to_le_bytes());
        offset += 8usize;
    }
    buffer[offset..offset + 4usize].copy_from_slice(&slot.crc);
    buffer
}

impl WriteTo for Slot {
    /// Writes the serialized `Slot` to the given writer. May write partially.
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        buf.write(&get_buffer(self))
    }

    /// Writes the entire serialized `Slot` to the writer. Will retry until complete.
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&get_buffer(self))
    }
}
