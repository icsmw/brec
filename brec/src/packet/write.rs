use crate::payload::EncodedPayload;
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
    fn write<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
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
        if let Some((payload_header, payload_body)) = prepared_payload.as_ref() {
            let payload_header = payload_header.as_vec();
            let written = buf.write(&payload_header)?;
            if written < payload_header.len() {
                return Ok(total + written);
            }
            total += written;

            let written = buf.write(payload_body.as_slice())?;
            if written < payload_body.len() {
                return Ok(total + written);
            }
            total += written;
        }
        Ok(total)
    }

    /// Writes the entire packet to the stream, retrying until all parts are written.
    ///
    /// This includes:
    /// - the computed `PacketHeader`
    /// - each individual block
    /// - optional payload
    fn write_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
        header.write_all(buf)?;
        for blk in self.blocks.iter() {
            blk.write_all(buf)?;
        }
        if let Some((payload_header, payload_body)) = prepared_payload.as_ref() {
            buf.write_all(&payload_header.as_vec())?;
            buf.write_all(payload_body.as_slice())?;
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
    fn slices(
        &mut self,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<IoSlices<'_>> {
        let prepared_payload = if let Some(payload) = self.payload.as_ref() {
            Some(prepare_payload(payload, ctx)?)
        } else {
            None
        };
        let payload_len = prepared_payload
            .as_ref()
            .map(|(header, body)| (header.size() + body.len()) as u64)
            .unwrap_or(0);
        let blocks_len: u64 = self.blocks.iter().map(|blk| blk.size()).sum();
        let header =
            PacketHeader::from_lengths(blocks_len, payload_len, prepared_payload.is_some());
        let mut slices = IoSlices::default();
        let mut header_bytes: Vec<u8> = Vec::new();
        header.write_all(&mut header_bytes)?;
        slices.add_buffered(header_bytes);
        for blk in self.blocks.iter() {
            slices.append(blk.slices()?);
        }
        if let Some((payload_header, payload_body)) = prepared_payload {
            slices.add_buffered(payload_header.as_vec());
            match payload_body {
                EncodedPayload::Borrowed(bytes) => slices.add_slice(bytes),
                EncodedPayload::Owned(bytes) => slices.add_buffered(bytes),
            }
        }
        Ok(slices)
    }
}
