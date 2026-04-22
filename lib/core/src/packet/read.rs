use crate::*;
#[cfg(feature = "resilient")]
use brec_consts::{BLOCK_CRC_LEN, BLOCK_SIG_LEN, BLOCK_SIZE_FIELD_LEN};

/// Reads a complete `PacketDef` from a stream, including header, blocks, and optional payload.
///
/// This implementation **does not support partial reads** - if the header is successfully
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
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> ReadPacketFrom
    for PacketDef<B, P, Inner>
{
    fn read<T: std::io::Read>(
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let header = PacketHeader::read(buf)?;
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            let mut blocks = vec![0u8; header.blocks_len as usize];
            buf.read_exact(&mut blocks)?;
            let mut reader = std::io::Cursor::new(blocks);
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
            let header = <PayloadHeader as ReadFrom>::read(buf)?;
            let payload = <P as ExtractPayloadFrom<Inner>>::read(buf, &header, ctx)?;
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
/// - `PacketReadStatus::Success(packet)` - full packet successfully read.
/// - `PacketReadStatus::NotEnoughData(bytes)` - more data needed to complete the packet.
/// - `Error` - on decoding, CRC, signature, or logic errors.
///
/// # Stream behavior
/// Seeks forward to read the packet, and seeks back on early return or error.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadPacketFrom
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> Result<PacketReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let available = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        #[cfg(feature = "resilient")]
        let mut unrecognized = Vec::new();
        let packet_header = match <PacketHeader as TryReadFrom>::try_read(buf)? {
            ReadStatus::NotEnoughData(needed) => {
                return Ok(PacketReadStatus::NotEnoughData(needed));
            }
            ReadStatus::Success(header) => header,
        };
        let packet_size = PacketHeader::ssize() + packet_header.size;
        if packet_size > available {
            return Ok(PacketReadStatus::NotEnoughData(packet_size - available));
        }
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if packet_header.blocks_len > 0 {
            let mut iterations = 0;
            loop {
                match <B as TryReadFrom>::try_read(buf) {
                    Ok(ReadStatus::Success(blk)) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == packet_header.blocks_len {
                            break;
                        }
                    }
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Ok(PacketReadStatus::NotEnoughData(needed));
                    }
                    Err(err) => {
                        #[cfg(feature = "resilient")]
                        if let Error::SignatureDismatch(mut entry) = err {
                            let Some(body_len) = entry.len else {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(Error::ZeroLengthBlock);
                            };
                            if body_len == 0 {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(Error::InvalidLength);
                            }
                            let block_len = BLOCK_SIG_LEN as u64
                                + BLOCK_SIZE_FIELD_LEN as u64
                                + body_len
                                + BLOCK_CRC_LEN as u64;
                            let blocks_left = packet_header.blocks_len.saturating_sub(read);
                            if block_len > blocks_left {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(Error::InvalidLength);
                            }
                            entry.pos = Some(PacketHeader::ssize() + read);
                            buf.seek(std::io::SeekFrom::Current(block_len as i64))?;
                            read += block_len;
                            unrecognized.push(entry);
                            if read == packet_header.blocks_len {
                                break;
                            }
                            iterations += 1;
                            if iterations > MAX_BLOCKS_COUNT as usize {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(Error::MaxBlocksCount);
                            }
                            continue;
                        }
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
        if packet_header.payload {
            match <PayloadHeader as TryReadFrom>::try_read(buf)? {
                ReadStatus::Success(payload_header) => {
                    let payload_total =
                        payload_header.size() as u64 + payload_header.payload_len() as u64;
                    let packet_payload_left = packet_header.size - packet_header.blocks_len;
                    if payload_total > packet_payload_left {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Err(Error::InvalidLength);
                    }
                    match <P as TryExtractPayloadFrom<Inner>>::try_read(buf, &payload_header, ctx) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
                        Ok(ReadStatus::NotEnoughData(needed)) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Ok(PacketReadStatus::NotEnoughData(needed));
                        }
                        Err(err) => {
                            #[cfg(feature = "resilient")]
                            if let Error::SignatureDismatch(mut entry) = err {
                                let payload_len = payload_header.payload_len() as u64;
                                let payload_total = payload_len + payload_header.size() as u64;
                                let packet_payload_left =
                                    packet_header.size - packet_header.blocks_len;
                                if payload_total > packet_payload_left {
                                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                    return Err(Error::InvalidLength);
                                }
                                entry.pos =
                                    Some(PacketHeader::ssize() + packet_header.blocks_len + 1);
                                entry.len = Some(payload_len);
                                buf.seek(std::io::SeekFrom::Current(payload_len as i64))?;
                                unrecognized.push(entry);
                            } else {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(err);
                            }
                            #[cfg(not(feature = "resilient"))]
                            {
                                buf.seek(std::io::SeekFrom::Start(start_pos))?;
                                return Err(err);
                            }
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(Error::NotEnoughData(needed as usize));
                }
            }
        }
        #[cfg(feature = "resilient")]
        {
            Ok(PacketReadStatus::success(pkg, unrecognized))
        }
        #[cfg(not(feature = "resilient"))]
        {
            Ok(PacketReadStatus::success(pkg))
        }
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
/// - `PacketReadStatus::Success(packet)` - if all required data was read and validated.
/// - `PacketReadStatus::NotEnoughData(bytes)` - if more bytes are needed.
/// - `Error::MaxBlocksCount` - if the block limit is exceeded.
/// - Any decoding or CRC/signature errors.
///
/// # Notes
/// The header and block stream are parsed directly from the internal buffer. Payload data may be buffered or streamed depending on implementation.
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadPacketFromBuffered
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::BufRead>(
        reader: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> Result<PacketReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let bytes = reader.fill_buf()?;
        let available = bytes.len() as u64;
        if available < PacketHeader::ssize() {
            return Ok(PacketReadStatus::NotEnoughData(
                PacketHeader::ssize() - available,
            ));
        }
        let packet_header = PacketHeader::read_from_slice(bytes, false)?;
        let packet_size = PacketHeader::ssize() + packet_header.size;
        if packet_size > available {
            return Ok(PacketReadStatus::NotEnoughData(packet_size - available));
        }
        reader.consume(PacketHeader::ssize() as usize);
        #[cfg(feature = "resilient")]
        let mut unrecognized = Vec::new();
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if packet_header.blocks_len > 0 {
            let mut iterations = 0;
            loop {
                match <B as TryReadFromBuffered>::try_read(reader) {
                    Ok(ReadStatus::Success(blk)) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == packet_header.blocks_len {
                            break;
                        }
                    }
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        return Ok(PacketReadStatus::NotEnoughData(needed));
                    }
                    Err(err) => {
                        #[cfg(feature = "resilient")]
                        if let Error::SignatureDismatch(mut entry) = err {
                            let Some(body_len) = entry.len else {
                                return Err(Error::ZeroLengthBlock);
                            };
                            if body_len == 0 {
                                return Err(Error::InvalidLength);
                            }
                            let block_len = BLOCK_SIG_LEN as u64
                                + BLOCK_SIZE_FIELD_LEN as u64
                                + body_len
                                + BLOCK_CRC_LEN as u64;
                            let blocks_left = packet_header.blocks_len.saturating_sub(read);
                            if block_len > blocks_left {
                                return Err(Error::InvalidLength);
                            }
                            entry.pos = Some(PacketHeader::ssize() + read);
                            reader.consume(block_len as usize);
                            read += block_len;
                            unrecognized.push(entry);
                            if read == packet_header.blocks_len {
                                break;
                            }
                            iterations += 1;
                            if iterations > MAX_BLOCKS_COUNT as usize {
                                return Err(Error::MaxBlocksCount);
                            }
                            continue;
                        }
                        return Err(err);
                    }
                }
                iterations += 1;
                if iterations > MAX_BLOCKS_COUNT as usize {
                    return Err(Error::MaxBlocksCount);
                }
            }
        }
        if packet_header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(reader)? {
                ReadStatus::Success(payload_header) => {
                    let payload_total =
                        payload_header.size() as u64 + payload_header.payload_len() as u64;
                    let packet_payload_left = packet_header.size - packet_header.blocks_len;
                    if payload_total > packet_payload_left {
                        return Err(Error::InvalidLength);
                    }
                    reader.consume(payload_header.size());
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        reader,
                        &payload_header,
                        ctx,
                    ) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
                        Ok(ReadStatus::NotEnoughData(needed)) => {
                            return Ok(PacketReadStatus::NotEnoughData(needed));
                        }
                        Err(err) => {
                            #[cfg(feature = "resilient")]
                            if let Error::SignatureDismatch(mut entry) = err {
                                let payload_len = payload_header.payload_len() as u64;
                                let payload_total = payload_len + payload_header.size() as u64;
                                let packet_payload_left =
                                    packet_header.size - packet_header.blocks_len;
                                if payload_total > packet_payload_left {
                                    return Err(Error::InvalidLength);
                                }
                                entry.pos =
                                    Some(PacketHeader::ssize() + packet_header.blocks_len + 1);
                                entry.len = Some(payload_len);
                                reader.consume(payload_len as usize);
                                unrecognized.push(entry);
                            } else {
                                return Err(err);
                            }
                            #[cfg(not(feature = "resilient"))]
                            return Err(err);
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(needed as usize));
                }
            }
        }
        #[cfg(feature = "resilient")]
        {
            Ok(PacketReadStatus::success(pkg, unrecognized))
        }
        #[cfg(not(feature = "resilient"))]
        {
            Ok(PacketReadStatus::success(pkg))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ByteBlock, DefaultPayloadContext, Error, PacketDef, PacketHeader, PacketReadStatus,
        PayloadHeader, ReadPacketFrom, TryReadPacketFrom, TryReadPacketFromBuffered, WriteTo,
        tests::{TestBlock, TestPayload},
    };
    use std::io::{BufReader, Cursor, Seek};

    fn empty_packet_bytes() -> Vec<u8> {
        let header = PacketHeader::from_lengths(0, 0, false);
        let mut out = Vec::new();
        header.write_all(&mut out).expect("packet header write");
        out
    }

    #[test]
    fn packet_read_and_try_read_and_buffered_succeed_for_empty_packet() {
        let bytes = empty_packet_bytes();

        let mut cursor = Cursor::new(bytes.clone());
        let packet = <PacketDef<TestBlock, TestPayload, TestPayload> as ReadPacketFrom>::read(
            &mut cursor,
            &mut DefaultPayloadContext::default(),
        )
        .expect("read empty packet");
        assert!(packet.blocks.is_empty());
        assert!(packet.payload.is_none());

        let mut cursor = Cursor::new(bytes.clone());
        match <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFrom>::try_read(
            &mut cursor,
            &mut DefaultPayloadContext::default(),
        )
        .expect("try_read empty packet")
        {
            PacketReadStatus::Success(packet) => {
                #[cfg(feature = "resilient")]
                let packet = &packet.0;
                #[cfg(not(feature = "resilient"))]
                let packet = &packet;

                assert!(packet.blocks.is_empty());
                assert!(packet.payload.is_none());
            }
            PacketReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
        assert_eq!(
            cursor.stream_position().expect("stream_position"),
            PacketHeader::SIZE
        );

        let mut reader = BufReader::new(Cursor::new(bytes));
        match <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFromBuffered>::try_read(
            &mut reader,
            &mut DefaultPayloadContext::default(),
        )
        .expect("buffered try_read empty packet")
        {
            PacketReadStatus::Success(packet) => {
                #[cfg(feature = "resilient")]
                let packet = &packet.0;
                #[cfg(not(feature = "resilient"))]
                let packet = &packet;

                assert!(packet.blocks.is_empty());
                assert!(packet.payload.is_none());
            }
            PacketReadStatus::NotEnoughData(_) => panic!("expected Success"),
        }
    }

    #[test]
    fn packet_try_read_and_buffered_report_not_enough_for_short_header() {
        let short = vec![1_u8, 2, 3];

        let mut cursor = Cursor::new(short.clone());
        match <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFrom>::try_read(
            &mut cursor,
            &mut DefaultPayloadContext::default(),
        )
        .expect("try_read short must not fail")
        {
            PacketReadStatus::NotEnoughData(needed) => assert!(needed > 0),
            PacketReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }
        assert_eq!(cursor.stream_position().expect("stream_position"), 0);

        let mut reader = BufReader::new(Cursor::new(short));
        match <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFromBuffered>::try_read(
            &mut reader,
            &mut DefaultPayloadContext::default(),
        )
        .expect("buffered try_read short must not fail")
        {
            PacketReadStatus::NotEnoughData(needed) => assert!(needed > 0),
            PacketReadStatus::Success(_) => panic!("expected NotEnoughData"),
        }
    }

    #[test]
    fn packet_try_read_and_buffered_detect_invalid_payload_length_mismatch() {
        let packet_header = PacketHeader::from_lengths(0, 0, true);
        let payload_header = PayloadHeader {
            sig: ByteBlock::Len4(*b"ABCD"),
            crc: ByteBlock::Len4([1, 2, 3, 4]),
            len: 1,
        };

        let mut bytes = Vec::new();
        packet_header
            .write_all(&mut bytes)
            .expect("packet header write");
        bytes.extend_from_slice(&payload_header.as_vec());

        let mut cursor = Cursor::new(bytes.clone());
        assert!(matches!(
            <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFrom>::try_read(
                &mut cursor,
                &mut DefaultPayloadContext::default(),
            ),
            Err(Error::InvalidLength)
        ));
        assert_eq!(
            cursor.stream_position().expect("stream_position"),
            0,
            "seekable try_read should reset position on error"
        );

        let mut reader = BufReader::new(Cursor::new(bytes));
        assert!(matches!(
            <PacketDef<TestBlock, TestPayload, TestPayload> as TryReadPacketFromBuffered>::try_read(
                &mut reader,
                &mut DefaultPayloadContext::default(),
            ),
            Err(Error::InvalidLength)
        ));
    }
}
