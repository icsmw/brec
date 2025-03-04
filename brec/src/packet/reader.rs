use std::io::BufRead;

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
    Ignored,

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
    Found(PacketHeader, std::ops::RangeInclusive<usize>),
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
pub struct PacketBufReaderDef<
    'a,
    R: std::io::Read,
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    /// Buffered reader for handling input stream operations.
    inner: std::io::BufReader<&'a mut R>,
    /// Collection of processing rules applied to incoming data.
    rules: RulesDef<W, B, BR, P, Inner>,
    /// Stores the current state of the header reading process.
    recent: HeaderReadState,
    /// Internal buffer for accumulating data before processing.
    buffered: Vec<u8>,
}

impl<
        'a,
        R: std::io::Read,
        W: std::io::Write,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > PacketBufReaderDef<'a, R, W, B, BR, P, Inner>
{
    fn read_header(buffer: &[u8]) -> Result<PacketHeaderState, Error> {
        let Some(offset) = PacketHeader::get_pos(buffer) else {
            // Signature of PacketDef isn't found
            return Ok(PacketHeaderState::NotFound);
        };
        if let Some(needed) = PacketHeader::is_not_enought(&buffer[offset..]) {
            // Header is detected, but not enough data to load it
            return Ok(PacketHeaderState::NotEnoughData(offset, needed));
        }
        Ok(PacketHeaderState::Found(
            PacketHeader::read_from_slice(&buffer[offset..], false)?,
            std::ops::RangeInclusive::new(offset, offset + PacketHeader::ssize() as usize),
        ))
    }

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
        let rest_data = packet_size - self.buffered.len();
        // Copy and consume only needed data
        self.buffered.extend_from_slice(&buffer[..rest_data]);
        self.inner.consume(rest_data);
        Ok(ResolveHeaderReady::Resolved(header))
    }

    fn resolve_header_refill(
        &mut self,
        mut buffer: Vec<u8>,
        needed: usize,
    ) -> Result<NextPacket<B, P, Inner>, Error> {
        let extracted = self.inner.fill_buf()?;
        if extracted.is_empty() {
            return Ok(NextPacket::NoData);
        }
        let extracted_len = extracted.len();
        if extracted_len < needed {
            buffer.extend_from_slice(extracted);
            self.inner.consume(extracted_len);
            self.recent = HeaderReadState::Refill(Some((buffer, needed - extracted_len)));
            return Ok(NextPacket::NotEnoughData(needed - extracted_len));
        }
        buffer.extend_from_slice(&extracted[..needed]);
        self.inner.consume(needed);
        match PacketBufReaderDef::<'a, R, W, B, BR, P, Inner>::read_header(&buffer)? {
            PacketHeaderState::Found(header, _sgmt) => {
                self.recent = HeaderReadState::Ready(Some(header));
                Ok(NextPacket::NotEnoughData(0))
            }
            _ => Err(Error::FailToReadPacketHeader),
        }
    }

    /// Creates a new instance of the reader.
    pub fn new(inner: &'a mut R) -> Self {
        Self {
            inner: std::io::BufReader::new(inner),
            rules: RulesDef::default(),
            recent: HeaderReadState::Empty,
            buffered: Vec::with_capacity(u32::MAX as usize),
        }
    }

    /// Adds a processing rule. See `RuleDef` for more details.
    pub fn add_rule(&mut self, rule: RuleDef<W, B, BR, P, Inner>) -> Result<(), Error> {
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
    pub fn read(&mut self) -> Result<NextPacket<B, P, Inner>, Error> {
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
                    let mut data: Vec<u8> = Vec::with_capacity(buffer.len());
                    data.extend_from_slice(buffer);
                    self.recent = HeaderReadState::Refill(Some((data, needed)));
                    self.inner.consume(available);
                    return Ok(NextPacket::NotEnoughData(needed));
                }
                match PacketBufReaderDef::<'a, R, W, B, BR, P, Inner>::read_header(buffer)? {
                    PacketHeaderState::NotFound => {
                        // Nothing found
                        self.rules.ignore(buffer)?;
                        // Consume all ignored data
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
        let referred = PacketReferred::new(blocks, header);
        self.rules.map(&referred)?;
        if !self.rules.filter(&referred) {
            // PacketDef marked as ignored
            return self.drop_and_consume(consume, Ok(NextPacket::Ignored));
        }
        let blocks = referred
            .blocks
            .into_iter()
            .map(|blk| blk.into())
            .collect::<Vec<B>>();
        let mut pkg: PacketDef<B, P, Inner> = PacketDef::new(blocks, None);
        let header = referred.header;
        // Loading payload if exists
        if header.payload {
            let mut payload_buffer = &packet_buffer[blocks_len..];
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut payload_buffer) {
                Ok(ReadStatus::Success(header)) => {
                    let mut payload_buffer = &packet_buffer[blocks_len + header.size()..];
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut payload_buffer,
                        &header,
                    )? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
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
        }
        self.drop_and_consume(consume, Ok(NextPacket::Found(pkg)))
    }
}
