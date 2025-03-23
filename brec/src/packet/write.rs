use crate::*;

/// Implements mutable stream writing for a full `PacketDef`, including header, blocks, and payload.
///
/// This trait supports writing packets to a stream with partial write handling
/// (`write`) and guaranteed complete writes (`write_all`).
///
/// # Behavior
/// - The `PacketHeader` is constructed on the fly based on the current blocks and payload.
/// - `write()` may return early if only part of the data was written.
/// - `write_all()` retries until all data is successfully written.
///
/// # Errors
/// Returns any I/O error or encoding failure from `BlockDef`, `PayloadDef`, or `PayloadHeader`.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteMutTo
    for PacketDef<B, P, Inner>
{
    /// Writes the packet to a stream, allowing for partial write detection.
    ///
    /// # Returns
    /// Total number of bytes written. May be less than full packet size if the underlying
    /// writer cannot accept all data in one go.
    ///
    /// Use `write_all()` if full delivery is required.
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        let mut total = header.write(buf)?;
        if total < PacketHeader::SIZE as usize {
            return Ok(total);
        }
        for blk in self.blocks.iter() {
            let size = blk.size() as usize;
            let written = blk.write(buf)?;
            if written < size {
                return Ok(total + written);
            }
            total += written;
        }
        if let Some(payload) = self.payload.as_mut() {
            let written = payload.write(buf)?;
            if written < payload.size()? as usize + PayloadHeader::ssize(payload)? {
                return Ok(total + written);
            } else {
                total += written
            }
        }
        Ok(total)
    }

    /// Writes the entire packet to the stream, retrying until all parts are written.
    ///
    /// This includes:
    /// - the computed `PacketHeader`
    /// - each individual block
    /// - optional payload
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        header.write_all(buf)?;
        for blk in self.blocks.iter() {
            blk.write_all(buf)?;
        }
        if let Some(payload) = self.payload.as_mut() {
            payload.write_all(buf)?;
        }
        Ok(())
    }
}

/// Implements vectored I/O writing for `PacketDef` using `IoSlices`.
///
/// This trait allows the entire packet to be described as a collection of contiguous slices,
/// which can then be written efficiently using `write_vectored()` or `write_vectored_all()`.
///
/// # Behavior
/// - Builds a dynamic `IoSlices` buffer including header, blocks, and optional payload.
/// - Encodes the header into a temporary buffer and adds it as the first slice.
/// - Calls `slices()` on each block and the payload (if present).
///
/// # Errors
/// Returns an error if header construction, encoding, or slice generation fails.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteVectoredMutTo
    for PacketDef<B, P, Inner>
{
    /// Returns an `IoSlices` collection representing the full serialized packet.
    ///
    /// This includes:
    /// - Serialized header (as buffered bytes)
    /// - Serialized block slices
    /// - Serialized payload slices (if any)
    ///
    /// # Returns
    /// A ready-to-write `IoSlices` that can be passed to `write_vectored`.
    fn slices(&mut self) -> std::io::Result<IoSlices> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        let mut slices = IoSlices::default();
        let mut header_bytes: Vec<u8> = Vec::new();
        header.write_all(&mut header_bytes)?;
        slices.add_buffered(header_bytes);
        for blk in self.blocks.iter() {
            slices.append(blk.slices()?);
        }
        if let Some(payload) = self.payload.as_mut() {
            slices.append(payload.slices()?);
        }
        Ok(slices)
    }
}
