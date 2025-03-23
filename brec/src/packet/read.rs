use crate::*;

/// Reads a complete `PacketDef` from a stream, including header, blocks, and optional payload.
///
/// This implementation **does not support partial reads** — if the header is successfully
/// read but the blocks or payload data are incomplete, an I/O error will be returned.
///
/// # Notes
/// - Does **not** return `Error::NotEnoughData`; instead, read failures always result in `std::io::Error`.
/// - Use this implementation when you're sure the entire packet is available in the stream.
///
/// # Errors
/// - `Error::SignatureDismatch` or `Error::CrcDismatch` if header validation fails.
/// - `Error::NotEnoughData` if there’s insufficient data in the inner block stream.
/// - `Error::MaxBlocksCount` if the block count exceeds the allowed maximum.
/// - Any decoding or payload-related error from underlying implementations.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> ReadFrom
    for PacketDef<B, P, Inner>
{
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let header = PacketHeader::read(buf)?;
        let mut pkg = PacketDef::default();
        let mut read = 0;
        let mut inner = vec![0u8; header.size as usize];
        buf.read_exact(&mut inner)?;
        let mut reader = std::io::Cursor::new(&mut inner);
        if header.blocks_len > 0 {
            let mut iterations = 0;
            loop {
                match <B as TryReadFromBuffered>::try_read(&mut reader)? {
                    ReadStatus::Success(blk) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    ReadStatus::NotEnoughData(needed) => {
                        return Err(Error::NotEnoughData(needed as usize));
                    }
                }
                iterations += 1;
                if iterations > MAX_BLOCKS_COUNT as usize {
                    return Err(Error::MaxBlocksCount);
                }
            }
        }
        if header.payload {
            let header = <PayloadHeader as ReadFrom>::read(&mut reader)?;
            let payload = <P as ExtractPayloadFrom<Inner>>::read(&mut reader, &header)?;
            pkg.payload = Some(payload);
        }
        Ok(pkg)
    }
}

/// Attempts to read a `PacketDef` from a seekable stream with partial read awareness.
///
/// This implementation checks if enough data is available before attempting to decode,
/// and can return `ReadStatus::NotEnoughData(...)` instead of failing with an I/O error.
///
/// # Behavior
/// - If not enough data is available for the entire payload, stream position is reset.
/// - If read fails partway through (block or payload), stream position is reset and the error returned.
/// - If block count exceeds `MAX_BLOCKS_COUNT`, returns `Error::MaxBlocksCount`.
///
/// # Returns
/// - `ReadStatus::Success(packet)` — full packet successfully read.
/// - `ReadStatus::NotEnoughData(bytes)` — more data needed to complete the packet.
/// - `Error` — on decoding, CRC, signature, or logic errors.
///
/// # Stream behavior
/// Seeks forward to read the packet, and seeks back on early return or error.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFrom
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let available = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        let header = match <PacketHeader as TryReadFrom>::try_read(buf)? {
            ReadStatus::NotEnoughData(needed) => return Ok(ReadStatus::NotEnoughData(needed)),
            ReadStatus::Success(header) => header,
        };
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            let mut iterations = 0;
            loop {
                match <B as TryReadFrom>::try_read(buf) {
                    Ok(ReadStatus::Success(blk)) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Ok(ReadStatus::NotEnoughData(needed));
                    }
                    Err(err) => {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Err(err);
                    }
                }
                iterations += 1;
                if iterations > MAX_BLOCKS_COUNT as usize {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(Error::MaxBlocksCount);
                }
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFrom>::try_read(buf)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFrom<Inner>>::try_read(buf, &header) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
                        Ok(ReadStatus::NotEnoughData(needed)) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Ok(ReadStatus::NotEnoughData(needed));
                        }
                        Err(err) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Err(err);
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(Error::NotEnoughData(needed as usize));
                }
            }
        }
        Ok(ReadStatus::Success(pkg))
    }
}

/// Attempts to read a `PacketDef` from a buffered reader.
///
/// This is similar to `TryReadFrom`, but works with non-seekable buffered sources (e.g., network streams).
///
/// # Behavior
/// - Reads header directly from `BufRead::fill_buf()` and consumes it.
/// - Ensures that `header.size` bytes are available before decoding.
/// - Supports partial reads using `ReadStatus::NotEnoughData(...)`.
///
/// # Returns
/// - `ReadStatus::Success(packet)` — if all required data was read and validated.
/// - `ReadStatus::NotEnoughData(bytes)` — if more bytes are needed.
/// - `Error::MaxBlocksCount` — if the block limit is exceeded.
/// - Any decoding or CRC/signature errors.
///
/// # Notes
/// The header and block stream are parsed directly from the internal buffer. Payload data may be buffered or streamed depending on implementation.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFromBuffered
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::BufRead>(reader: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let bytes = reader.fill_buf()?;
        let available = bytes.len() as u64;
        if available < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - available));
        }
        let header = PacketHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        reader.consume(PacketHeader::ssize() as usize);
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            let mut iterations = 0;
            loop {
                match <B as TryReadFromBuffered>::try_read(reader)? {
                    ReadStatus::Success(blk) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    ReadStatus::NotEnoughData(needed) => {
                        return Ok(ReadStatus::NotEnoughData(needed))
                    }
                }
                iterations += 1;
                if iterations > MAX_BLOCKS_COUNT as usize {
                    return Err(Error::MaxBlocksCount);
                }
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(reader)? {
                ReadStatus::Success(header) => {
                    reader.consume(header.size());
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(reader, &header)? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Ok(ReadStatus::NotEnoughData(needed))
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(needed as usize))
                }
            }
        }
        Ok(ReadStatus::Success(pkg))
    }
}
