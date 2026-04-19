use std::{io::BufRead,ops::RangeInclusive};

use crate::*;

/// Represents the result of reading from `PacketBufReaderDef`.
pub enum NextPacket<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    /// Not enough data available to read the current packet.
    /// This does not necessarily mean that a `brec` packet was detected but rather that
    /// additional data is required to determine its presence.
    ///
    /// `PacketBufReaderDef` may also return `NotEnoughData(0)` if a previous iteration
    /// was unable to read a `brec` header. In such cases, reading should continue.
    NotEnoughData(usize),

    /// Returned when `PacketBufReaderDef` is unable to obtain new data.
    /// If this result is received, there is no point in continuing the read operation
    /// as the data source appears to be exhausted.
    NoData,

    /// Returned if `PacketBufReaderDef` reads a segment of data but does not detect a `brec` packet.
    /// Reading may continue in the next iteration.
    ///
    /// If a non-`brec` segment is detected in the same iteration where a `brec` packet is found,
    /// `NoData` will not be returned. Instead, `NotEnoughData`, `Ignored`, or `Found` will be used.
    NotFound,

    /// Returned when `PacketBufReaderDef` recognizes a `brec` packet but it was ignored according to the rules.
    Skipped,

    /// Indicates successful parsing of a `brec` packet that has passed filtering rules (if any exist).
    Found(PacketDef<B, P, Inner>),
}
/// Internal structure used by `PacketBufReaderDef` when reading packet headers.
pub enum PacketHeaderState {
    /// Header was not found.
    NotFound,
    /// Not enough data available to read the header.
    NotEnoughData(usize, usize),
    /// Header was successfully read.
    ///
    /// Contains:
    /// - The parsed `PacketHeader`.
    /// - The position of the header within the provided data slice.
    Found(PacketHeader, RangeInclusive<usize>),
}

/// Internal structure used by `PacketBufReaderDef` when reading packet headers.
/// This structure is utilized in rare cases when there is insufficient data to read a header.
pub enum HeaderReadState {
    /// The header has been successfully read.
    Ready(Option<PacketHeader>),
    /// More data is required.
    ///
    /// Contains:
    /// - A buffer storing previously received data.
    /// - The size of the missing data required to complete the header read.
    Refill(Option<(Vec<u8>, usize)>),
    /// Default state. Indicates that no additional data loading is required for reading a header.
    Empty,
}

/// Internal structure used by `PacketBufReaderDef` for handling packet header resolution.
pub enum ResolveHeaderReady<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    /// Indicates that the next action should be taken in processing.
    Next(NextPacket<B, P, Inner>),
    /// The packet header has been successfully resolved.
    Resolved(PacketHeader),
}

/// A stream reader for extracting `brec` packets.
///
/// `PacketBufReaderDef` supports reading from both "pure" streams containing only `brec` packets
/// and mixed streams where `brec` packets are interspersed with other data. The `Rules` mechanism
/// allows users to handle non-`brec` data instead of discarding it, enabling logging or reprocessing
/// if necessary.
///
/// It is important to note that there is no need to use `PacketBufReaderDef` directly.
/// When invoking the `generate!()` macro, a wrapper type `PacketBufReader<R: std::io::Read, W: std::io::Write>`
/// is generated, eliminating the requirement to specify all generic parameters manually. Users should
/// prefer `PacketBufReader` when creating a reader instance.
///
/// The generic parameters `<B, BR, P, Inner>` are automatically bound to the `Block` and `Payload`
/// types defined by the user using the corresponding `#[block]` and `#[payload]` macros.
/// This abstraction frees the user from the need to explicitly propagate these types.
pub struct PacketBufReaderDef<
    'a,
    R: std::io::Read,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    /// Buffered reader for handling input stream operations.
    inner: std::io::BufReader<&'a mut R>,
    /// Collection of processing rules applied to incoming data.
    rules: RulesDef<B, BR, P, Inner>,
    /// Stores the current state of the header reading process.
    recent: HeaderReadState,
    /// Internal buffer for accumulating data before processing.
    buffered: Vec<u8>,
}

impl<
    'a,
    R: std::io::Read,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> PacketBufReaderDef<'a, R, B, BR, P, Inner>
{
    /// Parses a packet header from the provided buffer.
    ///
    /// This function attempts to locate the `brec` packet signature in the given byte slice.
    /// If a signature is found but there is insufficient data to parse the full header,
    /// it returns `PacketHeaderState::NotEnoughData`. If the header is successfully parsed,
    /// it returns `PacketHeaderState::Found`.
    fn read_header(buffer: &[u8]) -> Result<PacketHeaderState, Error> {
        let mut first_not_enough: Option<(usize, usize)> = None;
        let mut base = 0usize;
        while base < buffer.len() {
            let Some(relative) = PacketHeader::get_pos(&buffer[base..]) else {
                break;
            };
            let offset = base + relative;
            if let Some(needed) = PacketHeader::is_not_enought(&buffer[offset..]) {
                if first_not_enough.is_none() {
                    first_not_enough = Some((offset, needed));
                }
                base = offset + 1;
                continue;
            }
            match PacketHeader::read_from_slice(&buffer[offset..], false) {
                Ok(header) => {
                    return Ok(PacketHeaderState::Found(
                        header,
                        RangeInclusive::new(offset, offset + PacketHeader::ssize() as usize),
                    ));
                }
                // Litter can accidentally contain a packet signature; continue searching.
                Err(Error::SignatureDismatch(_)) | Err(Error::CrcDismatch) => {
                    base = offset + 1;
                }
                Err(err) => return Err(err),
            }
        }
        if let Some((from, needed)) = first_not_enough {
            Ok(PacketHeaderState::NotEnoughData(from, needed))
        } else {
            // Signature of PacketDef isn't found
            Ok(PacketHeaderState::NotFound)
        }
    }

    /// Clears the internal buffer and consumes a specified number of bytes from the reader.
    ///
    /// This function is used to maintain the correct reading position while ensuring
    /// that previously processed data does not interfere with subsequent reads.
    fn drop_and_consume(
        &mut self,
        consume: Option<usize>,
        result: Result<NextPacket<B, P, Inner>, Error>,
    ) -> Result<NextPacket<B, P, Inner>, Error> {
        self.buffered.clear();
        if let Some(s) = consume {
            self.inner.consume(s)
        }
        result
    }

    /// Attempts to process a previously detected header when sufficient data is available.
    ///
    /// If enough data is present in the buffer, the method confirms the presence of the packet
    /// and returns `ResolveHeaderReady::Resolved`. Otherwise, it buffers additional data and
    /// signals `ResolveHeaderReady::Next` with `NotEnoughData`.
    fn resolve_header_ready(
        &mut self,
        header: PacketHeader,
    ) -> Result<ResolveHeaderReady<B, P, Inner>, Error> {
        let buffer = self.inner.fill_buf()?;
        // Check do we have enough data to load packet
        let packet_size = header.size as usize;
        let available = self.buffered.len() + buffer.len();
        if packet_size > available {
            // Not enough data to load packet
            let consumed = buffer.len();
            self.buffered.extend_from_slice(buffer);
            self.inner.consume(consumed);
            self.recent = HeaderReadState::Ready(Some(header));
            return Ok(ResolveHeaderReady::Next(NextPacket::NotEnoughData(
                packet_size - available,
            )));
        }
        if packet_size < self.buffered.len() {
            return Err(Error::InvalidPacketReaderLogic);
        }
        let rest_data = packet_size - self.buffered.len();
        // Copy and consume only needed data
        self.buffered.extend_from_slice(&buffer[..rest_data]);
        self.inner.consume(rest_data);
        Ok(ResolveHeaderReady::Resolved(header))
    }

    /// Processes buffered data when more input is required to complete header parsing.
    ///
    /// This method is used in cases where an incomplete header was previously encountered.
    /// It attempts to read additional data and, if successful, resumes header parsing.
    fn resolve_header_refill(
        &mut self,
        mut buffer: Vec<u8>,
        needed: usize,
    ) -> Result<NextPacket<B, P, Inner>, Error> {
        let read_header = PacketBufReaderDef::<'a, R, B, BR, P, Inner>::read_header;
        let extracted = self.inner.fill_buf()?;
        let extracted_len = extracted.len();
        let buffered = buffer.len();
        let mut appended = extracted_len.min(needed);
        if appended > 0 {
            // First make attempt to read header with just enough newly received bytes.
            buffer.extend_from_slice(&extracted[..appended]);
        }
        let mut status = read_header(&buffer)?;
        if !matches!(status, PacketHeaderState::Found(_, _)) && appended < extracted_len {
            // If not found yet, include all currently available bytes and re-check.
            buffer.extend_from_slice(&extracted[appended..]);
            appended = extracted_len;
            status = read_header(&buffer)?;
        }
        let header_len = PacketHeader::ssize() as usize;
        match status {
            PacketHeaderState::Found(header, sgmt) => {
                let header_start = *sgmt.start();
                let header_end = *sgmt.end();
                if header_start > 0 {
                    self.rules.ignore(&buffer[..header_start])?;
                }
                let consumed_from_extracted = header_end.saturating_sub(buffered).min(appended);
                let consumed_front = buffered + consumed_from_extracted;
                let payload_prefetched = consumed_front
                    .saturating_sub(header_end)
                    .min(header.size as usize);
                let packet_end = header_end + header.size as usize;
                let ignored_tail_end = consumed_front.min(buffer.len());
                if packet_end < ignored_tail_end {
                    self.rules.ignore(&buffer[packet_end..ignored_tail_end])?;
                }
                self.buffered.clear();
                if payload_prefetched > 0 {
                    self.buffered.extend_from_slice(
                        &buffer[header_end..header_end + payload_prefetched],
                    );
                }
                self.inner.consume(consumed_from_extracted);
                self.recent = HeaderReadState::Ready(Some(header));
                Ok(NextPacket::NotEnoughData(0))
            }
            PacketHeaderState::NotEnoughData(from, needed) => {
                if appended == 0 {
                    self.rules.ignore(&buffer)?;
                    return Ok(NextPacket::NoData);
                }
                if from > 0 {
                    // We can drain most bytes in buffer and left only length of header signature
                    self.rules.ignore(&buffer[..from])?;
                    buffer.drain(..from);
                }
                self.inner.consume(appended);
                self.recent = HeaderReadState::Refill(Some((buffer, needed)));
                Ok(NextPacket::NotEnoughData(needed))
            }
            PacketHeaderState::NotFound => {
                if appended == 0 {
                    self.rules.ignore(&buffer)?;
                    return Ok(NextPacket::NoData);
                }
                if buffer.len() > header_len {
                    self.rules.ignore(&buffer[..buffer.len() - header_len])?;
                    // We can drain most bytes in buffer and left only length of header signature
                    buffer.drain(..(buffer.len() - header_len));
                }
                self.inner.consume(appended);
                self.recent = HeaderReadState::Refill(Some((buffer, header_len)));
                Ok(NextPacket::NotFound)
            }
        }
    }

    /// Creates a new instance of the reader with explicit options.
    pub fn new(inner: &'a mut R) -> Self {
        Self {
            inner: std::io::BufReader::new(inner),
            rules: RulesDef::default(),
            recent: HeaderReadState::Empty,
            buffered: Vec::with_capacity(u32::MAX as usize),
        }
    }

    /// Adds a processing rule. See `RuleDef` for more details.
    pub fn add_rule(&mut self, rule: RuleDef<B, BR, P, Inner>) -> Result<(), Error> {
        self.rules.add_rule(rule)
    }

    /// Removes a previously added rule. See `RuleDef` for more details.
    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules.remove_rule(rule);
    }

    /// Reads the current portion of data available in the internal `BufReader`.
    ///
    /// This method does **not** invoke `read` or otherwise fetch additional data into the internal buffer.
    /// Instead, it processes only the existing buffered data. However, `consume` **will be called** to advance
    /// the read position.
    ///
    /// To continue reading, the user must call `read` on `PacketBufReaderDef` again to process more data.
    pub fn read(
        &mut self,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<NextPacket<B, P, Inner>, Error> {
        let recent = std::mem::replace(&mut self.recent, HeaderReadState::Empty);
        let (packet_buffer, header, consume) = match recent {
            HeaderReadState::Ready(Some(header)) => match self.resolve_header_ready(header)? {
                ResolveHeaderReady::Next(next) => return Ok(next),
                ResolveHeaderReady::Resolved(header) => (self.buffered.as_slice(), header, None),
            },
            HeaderReadState::Refill(Some((buffer, needed))) => {
                return self.resolve_header_refill(buffer, needed);
            }
            HeaderReadState::Empty => {
                self.buffered.clear();
                let buffer = self.inner.fill_buf()?;
                if buffer.is_empty() {
                    return Ok(NextPacket::NoData);
                }
                let available = buffer.len();
                if available < PacketHeader::ssize() as usize {
                    let needed = (PacketHeader::ssize() as usize) - available;
                    let mut data: Vec<u8> = Vec::with_capacity(available);
                    data.extend_from_slice(buffer);
                    self.recent = HeaderReadState::Refill(Some((data, needed)));
                    self.inner.consume(available);
                    return Ok(NextPacket::NotEnoughData(needed));
                }
                match PacketBufReaderDef::<'a, R, B, BR, P, Inner>::read_header(buffer)? {
                    PacketHeaderState::NotFound => {
                        let header_len = PacketHeader::ssize() as usize;
                        if available > header_len {
                            self.rules.ignore(&buffer[..available - header_len])?;
                            self.recent = HeaderReadState::Refill(Some((
                                buffer[available - header_len..].to_vec(),
                                header_len,
                            )));
                        } else {
                            self.recent =
                                HeaderReadState::Refill(Some((buffer.to_vec(), header_len)));
                        }
                        self.inner.consume(available);
                        return Ok(NextPacket::NotFound);
                    }
                    PacketHeaderState::NotEnoughData(from, needed) => {
                        // Not enough data to read packet header
                        if from > 0 {
                            self.rules.ignore(&buffer[..from])?;
                        }
                        let mut data: Vec<u8> = Vec::with_capacity(buffer.len() - from);
                        data.extend_from_slice(&buffer[from..]);
                        self.recent = HeaderReadState::Refill(Some((data, needed)));
                        self.inner.consume(available);
                        return Ok(NextPacket::NotEnoughData(needed));
                    }
                    PacketHeaderState::Found(header, sgmt) => {
                        // PacketDef header has been found
                        if sgmt.start() > &0 {
                            self.rules.ignore(&buffer[..*sgmt.start()])?;
                        }
                        let packet_size = header.size as usize;
                        let needs = packet_size + *sgmt.end();
                        if needs > available {
                            // Not enough data to load packet
                            self.buffered.extend_from_slice(&buffer[*sgmt.end()..]);
                            self.inner.consume(available);
                            self.recent = HeaderReadState::Ready(Some(header));
                            return Ok(NextPacket::NotEnoughData(needs - available));
                        }
                        let consume = Some(*sgmt.end() + header.size as usize);
                        (
                            &buffer[*sgmt.end()..*sgmt.end() + header.size as usize],
                            header,
                            consume,
                        )
                    }
                }
            }
            _error => {
                // We cannot be in this situation, because recent switched to
                // HeaderReadState::Empty by default
                return Err(Error::InvalidPacketReaderLogic);
            }
        };
        let blocks_len = header.blocks_len as usize;
        let blocks_buffer = &packet_buffer[..blocks_len];
        let mut blocks = Vec::new();
        let mut processed = 0;
        let mut count = 0;
        if !blocks_buffer.is_empty() {
            loop {
                if count == MAX_BLOCKS_COUNT {
                    self.buffered.clear();
                    return Err(Error::MaxBlocksCount);
                }
                let blk = match BR::read_from_slice(&blocks_buffer[processed..], false) {
                    Ok(blk) => blk,
                    Err(err) => {
                        return self.drop_and_consume(consume, Err(err));
                    }
                };
                if blk.size() == 0 {
                    return self.drop_and_consume(consume, Err(Error::ZeroLengthBlock));
                }
                processed += blk.size() as usize;
                count += 1;
                blocks.push(blk);
                if processed == blocks_buffer.len() {
                    break;
                }
            }
        }
        if !self.rules.prefilter(&blocks) {
            // PacketDef marked as ignored
            return self.drop_and_consume(consume, Ok(NextPacket::Skipped));
        }
        // Loading payload if exists
        let pkg = if header.payload {
            let mut payload_buffer = &packet_buffer[blocks_len..];
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut payload_buffer) {
                Ok(ReadStatus::Success(header)) => {
                    let mut payload_buffer = &packet_buffer[blocks_len + header.size()..];
                    if !self.rules.filter_payload(payload_buffer) {
                        // PacketDef marked as ignored
                        return self.drop_and_consume(consume, Ok(NextPacket::Skipped));
                    }
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut payload_buffer,
                        &header,
                        ctx,
                    )? {
                        ReadStatus::Success(payload) => PacketDef::new(
                            blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                            Some(payload),
                        ),
                        ReadStatus::NotEnoughData(needed) => {
                            // This is error, but not NextPacket::NotEnoughData because length of payload
                            // already has been check. If we are here - some data is invalid and
                            // it's an error
                            return self.drop_and_consume(
                                consume,
                                Err(Error::NotEnoughData(needed as usize)),
                            );
                        }
                    }
                }
                Ok(ReadStatus::NotEnoughData(needed)) => {
                    // This is error, but not NextPacket::NotEnoughData because length of payload
                    // already has been check. If we are here - some data is invalid and
                    // it's an error
                    return self
                        .drop_and_consume(consume, Err(Error::NotEnoughData(needed as usize)));
                }
                Err(err) => {
                    return self.drop_and_consume(consume, Err(err));
                }
            }
        } else {
            PacketDef::new(
                blocks.into_iter().map(|blk| blk.into()).collect::<Vec<B>>(),
                None,
            )
        };
        if !self.rules.filter_packet(&pkg) {
            // PacketDef marked as ignored
            self.drop_and_consume(consume, Ok(NextPacket::Skipped))
        } else {
            self.drop_and_consume(consume, Ok(NextPacket::Found(pkg)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuleDef, RuleFnDef, tests::*};
    use std::io::Cursor;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    type ReaderUnderTest<'a> =
        PacketBufReaderDef<'a, Cursor<Vec<u8>>, TestBlock, TestBlock, TestPayload, TestPayload>;

    fn empty_packet_bytes() -> Vec<u8> {
        let header = PacketHeader::from_lengths(0, 0, false);
        let mut out = Vec::new();
        header.write_all(&mut out).expect("packet header write");
        out
    }

    #[test]
    fn read_header_handles_not_found_not_enough_and_found() {
        assert!(matches!(
            ReaderUnderTest::read_header(&[1, 2, 3, 4]).expect("read_header"),
            PacketHeaderState::NotFound
        ));

        let mut partial = empty_packet_bytes();
        partial.truncate((PacketHeader::ssize() as usize).saturating_sub(1));
        assert!(matches!(
            ReaderUnderTest::read_header(&partial).expect("read_header"),
            PacketHeaderState::NotEnoughData(_, _)
        ));

        let mut with_prefix = vec![9, 9, 9];
        with_prefix.extend_from_slice(&empty_packet_bytes());
        match ReaderUnderTest::read_header(&with_prefix).expect("read_header found") {
            PacketHeaderState::Found(_, range) => assert_eq!(*range.start(), 3),
            PacketHeaderState::NotFound | PacketHeaderState::NotEnoughData(_, _) => {
                panic!("expected found header")
            }
        }
    }

    #[test]
    fn read_skips_false_signature_in_litter_and_finds_real_packet() {
        let valid = empty_packet_bytes();
        let mut false_sig_tail = crate::PACKET_SIG.to_vec();
        false_sig_tail.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let mut input_bytes = vec![0x11, 0x22, 0x33];
        input_bytes.extend_from_slice(&false_sig_tail);
        input_bytes.extend_from_slice(&[0x44, 0x55, 0x66]);
        let litter_len = input_bytes.len();
        input_bytes.extend_from_slice(&valid);

        let mut input = Cursor::new(input_bytes);
        let mut reader = ReaderUnderTest::new(&mut input);

        let ignored = Arc::new(AtomicUsize::new(0));
        let ignored_c = ignored.clone();
        reader
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |bytes| {
                ignored_c.fetch_add(bytes.len(), Ordering::SeqCst);
            }))))
            .expect("ignored callback");

        match reader.read(&mut ()).expect("first read") {
            NextPacket::Found(packet) => {
                assert!(packet.blocks.is_empty());
                assert!(packet.payload.is_none());
            }
            NextPacket::NotEnoughData(_)
            | NextPacket::NoData
            | NextPacket::NotFound
            | NextPacket::Skipped => panic!("expected Found"),
        }
        assert_eq!(ignored.load(Ordering::SeqCst), litter_len);

        assert!(matches!(
            reader.read(&mut ()).expect("second read"),
            NextPacket::NoData
        ));
    }

    #[test]
    fn read_header_returns_first_not_enough_when_multiple_candidates_are_short() {
        let mut buffer = vec![0xA1, 0xA2];
        buffer.extend_from_slice(&crate::PACKET_SIG);
        buffer.extend_from_slice(&[0xB1, 0xB2, 0xB3]);
        buffer.extend_from_slice(&crate::PACKET_SIG);
        buffer.extend_from_slice(&[0xC1, 0xC2]);

        let first = PacketHeader::get_pos(&buffer).expect("first signature");
        let second = PacketHeader::get_pos(&buffer[first + 1..]).expect("second signature");
        let second = first + 1 + second;
        let expected = PacketHeader::ssize() as usize - (buffer.len() - first);

        // Phase 1: the first candidate is short, so reader reports NotEnoughData from it.
        match ReaderUnderTest::read_header(&buffer).expect("read_header not enough") {
            PacketHeaderState::NotEnoughData(from, needed) => {
                assert_eq!(from, first);
                assert_eq!(needed, expected);
                assert!(second > first);
            }
            PacketHeaderState::Found(_, _) | PacketHeaderState::NotFound => {
                panic!("expected NotEnoughData")
            }
        }

        // Phase 2: complete the second candidate into a valid header.
        let valid = empty_packet_bytes();
        let needed_len = second + valid.len();
        if buffer.len() < needed_len {
            buffer.resize(needed_len, 0);
        }
        buffer[second..second + valid.len()].copy_from_slice(&valid);

        // First candidate is ignored and second is accepted.
        match ReaderUnderTest::read_header(&buffer).expect("read_header found") {
            PacketHeaderState::Found(_, range) => {
                assert_eq!(*range.start(), second);
            }
            PacketHeaderState::NotFound | PacketHeaderState::NotEnoughData(_, _) => {
                panic!("expected Found")
            }
        }
    }

    #[test]
    fn read_refill_eof_keeps_valid_packet_from_buffer() {
        let valid = empty_packet_bytes();
        let before = vec![0xAB, 0xCD, 0xEF];
        let after = vec![0x42, 0x24];

        let mut refill_buffer = Vec::with_capacity(before.len() + valid.len() + after.len());
        refill_buffer.extend_from_slice(&before);
        refill_buffer.extend_from_slice(&valid);
        refill_buffer.extend_from_slice(&after);

        let mut input = Cursor::new(Vec::<u8>::new());
        let mut reader = ReaderUnderTest::new(&mut input);
        reader.recent = HeaderReadState::Refill(Some((refill_buffer, 11)));

        let ignored = Arc::new(AtomicUsize::new(0));
        let ignored_c = ignored.clone();
        reader
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |bytes| {
                ignored_c.fetch_add(bytes.len(), Ordering::SeqCst);
            }))))
            .expect("ignored callback");

        assert!(matches!(
            reader.read(&mut ()).expect("refill at eof"),
            NextPacket::NotEnoughData(0)
        ));
        assert_eq!(ignored.load(Ordering::SeqCst), before.len() + after.len());

        match reader.read(&mut ()).expect("packet from refill buffer") {
            NextPacket::Found(packet) => {
                assert!(packet.blocks.is_empty());
                assert!(packet.payload.is_none());
            }
            NextPacket::NotEnoughData(_)
            | NextPacket::NoData
            | NextPacket::NotFound
            | NextPacket::Skipped => panic!("expected Found"),
        }

        assert!(matches!(
            reader.read(&mut ()).expect("stream end"),
            NextPacket::NoData
        ));
    }

    #[test]
    fn read_refill_not_enough_with_prefix_ignores_prefix_and_consumes_new_data() {
        let mut refill_buffer = vec![0x11, 0x22];
        refill_buffer.extend_from_slice(&crate::PACKET_SIG);
        refill_buffer.push(0x33);

        let mut input = Cursor::new(vec![0x44, 0x55, 0x66, 0x77, 0x88]);
        let mut reader = ReaderUnderTest::new(&mut input);
        reader.recent = HeaderReadState::Refill(Some((refill_buffer, PacketHeader::ssize() as usize)));

        let ignored = Arc::new(AtomicUsize::new(0));
        let ignored_c = ignored.clone();
        reader
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |bytes| {
                ignored_c.fetch_add(bytes.len(), Ordering::SeqCst);
            }))))
            .expect("ignored callback");

        match reader.read(&mut ()).expect("refill read") {
            NextPacket::NotEnoughData(needed) => assert!(needed > 0),
            NextPacket::Found(_)
            | NextPacket::NoData
            | NextPacket::NotFound
            | NextPacket::Skipped => panic!("expected NotEnoughData"),
        }
        assert_eq!(ignored.load(Ordering::SeqCst), 2);
        assert_eq!(input.position() as usize, 5);
    }

    #[test]
    fn read_refill_not_found_with_new_data_keeps_only_signature_tail() {
        let initial = vec![0xAA, 0xBB, 0xCC];
        let header_len = PacketHeader::ssize() as usize;
        let extracted = vec![0x10; header_len + 5];

        let mut input = Cursor::new(extracted.clone());
        let mut reader = ReaderUnderTest::new(&mut input);
        reader.recent = HeaderReadState::Refill(Some((initial.clone(), header_len)));

        let ignored = Arc::new(AtomicUsize::new(0));
        let ignored_c = ignored.clone();
        reader
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |bytes| {
                ignored_c.fetch_add(bytes.len(), Ordering::SeqCst);
            }))))
            .expect("ignored callback");

        assert!(matches!(
            reader.read(&mut ()).expect("refill not found"),
            NextPacket::NotFound
        ));
        assert_eq!(
            ignored.load(Ordering::SeqCst),
            initial.len() + extracted.len() - header_len
        );

        assert!(matches!(
            reader.read(&mut ()).expect("eof after tail"),
            NextPacket::NoData
        ));
        assert_eq!(ignored.load(Ordering::SeqCst), initial.len() + extracted.len());
        drop(reader);
        assert_eq!(input.position() as usize, extracted.len());
    }

    #[test]
    fn read_preserves_full_ignored_bytes_sequence() {
        let packet = empty_packet_bytes();

        let litter_prefix = vec![0xA1, 0xB2, 0xC3, 0xD4];
        let mut litter_mid = vec![0x11, 0x22, 0x33];
        litter_mid.extend_from_slice(&crate::PACKET_SIG);
        litter_mid.extend_from_slice(&[0x44, 0x55, 0x66, 0x77]);
        let litter_suffix = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01];

        let mut stream = Vec::new();
        stream.extend_from_slice(&litter_prefix);
        stream.extend_from_slice(&packet);
        stream.extend_from_slice(&litter_mid);
        stream.extend_from_slice(&packet);
        stream.extend_from_slice(&litter_suffix);

        let mut expected_ignored = Vec::new();
        expected_ignored.extend_from_slice(&litter_prefix);
        expected_ignored.extend_from_slice(&litter_mid);
        expected_ignored.extend_from_slice(&litter_suffix);

        let mut input = Cursor::new(stream);
        let mut reader = ReaderUnderTest::new(&mut input);

        let ignored = Arc::new(Mutex::new(Vec::<u8>::new()));
        let ignored_c = ignored.clone();
        reader
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |bytes| {
                ignored_c
                    .lock()
                    .expect("ignored bytes lock")
                    .extend_from_slice(bytes);
            }))))
            .expect("ignored callback");

        let mut found = 0usize;
        loop {
            match reader.read(&mut ()).expect("reader read") {
                NextPacket::Found(packet) => {
                    assert!(packet.blocks.is_empty());
                    assert!(packet.payload.is_none());
                    found += 1;
                }
                NextPacket::NotEnoughData(_) | NextPacket::NotFound | NextPacket::Skipped => {}
                NextPacket::NoData => break,
            }
        }

        assert_eq!(found, 2);
        assert_eq!(
            *ignored.lock().expect("ignored bytes lock"),
            expected_ignored
        );
    }

    #[test]
    fn read_emits_found_for_empty_packet_then_no_data() {
        let mut input = Cursor::new(empty_packet_bytes());
        let mut reader = ReaderUnderTest::new(&mut input);

        match reader.read(&mut ()).expect("first read") {
            NextPacket::Found(packet) => {
                assert!(packet.blocks.is_empty());
                assert!(packet.payload.is_none());
            }
            NextPacket::NotEnoughData(_)
            | NextPacket::NoData
            | NextPacket::NotFound
            | NextPacket::Skipped => panic!("expected Found"),
        }

        assert!(matches!(
            reader.read(&mut ()).expect("second read"),
            NextPacket::NoData
        ));
    }

    #[test]
    fn read_short_header_then_end_returns_not_enough_then_no_data() {
        let short_len = PacketHeader::ssize() as usize - 2;
        let mut input = Cursor::new(vec![0xAB; short_len]);
        let mut reader = ReaderUnderTest::new(&mut input);

        match reader.read(&mut ()).expect("first read") {
            NextPacket::NotEnoughData(needed) => assert_eq!(needed, 2),
            NextPacket::NoData
            | NextPacket::NotFound
            | NextPacket::Skipped
            | NextPacket::Found(_) => {
                panic!("expected NotEnoughData")
            }
        }

        assert!(matches!(
            reader.read(&mut ()).expect("second read"),
            NextPacket::NoData
        ));
    }

    #[test]
    fn read_can_skip_packet_with_prefilter_rule() {
        let mut input = Cursor::new(empty_packet_bytes());
        let mut reader = ReaderUnderTest::new(&mut input);
        reader
            .add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| false)))
            .expect("prefilter rule");

        assert!(matches!(
            reader.read(&mut ()).expect("read with prefilter"),
            NextPacket::Skipped
        ));
    }

    #[test]
    fn add_rule_duplicate_then_remove_allows_readd() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut reader = ReaderUnderTest::new(&mut input);

        reader
            .add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| true)))
            .expect("first prefilter add");
        assert!(matches!(
            reader.add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| true))),
            Err(Error::RuleDuplicate)
        ));

        reader.remove_rule(RuleDefId::Prefilter);
        reader
            .add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| true)))
            .expect("prefilter re-add after remove");
    }
}
