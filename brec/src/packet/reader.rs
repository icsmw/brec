use std::{io::BufRead, ops::RangeInclusive};

use crate::*;

pub enum RuleFnDef<D, S> {
    Dynamic(D),
    Static(S),
}

pub type IgnoredCallback = RuleFnDef<Box<dyn Fn(&[u8])>, fn(&[u8])>;

pub type WriteIgnoredCallback<W> = RuleFnDef<
    Box<dyn Fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>>,
    fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>,
>;

pub type FilterCallback<B, BR, P, Inner> = RuleFnDef<
    Box<dyn Fn(&PacketReferred<B, BR, P, Inner>) -> bool>,
    fn(&PacketReferred<B, BR, P, Inner>) -> bool,
>;

pub type MapCallback<W, B, BR, P, Inner> = RuleFnDef<
    Box<
        dyn Fn(&mut std::io::BufWriter<W>, &PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>,
    >,
    fn(&mut std::io::BufWriter<W>, &PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>,
>;

#[enum_ids::enum_ids(display)]
pub enum RuleDef<
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    Ignored(IgnoredCallback),
    WriteIgnored(std::io::BufWriter<W>, WriteIgnoredCallback<W>),
    Filter(FilterCallback<B, BR, P, Inner>),
    Map(std::io::BufWriter<W>, MapCallback<W, B, BR, P, Inner>),
}

pub struct RulesDef<
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub rules: Vec<RuleDef<W, B, BR, P, Inner>>,
}

impl<
        W: std::io::Write,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Default for RulesDef<W, B, BR, P, Inner>
{
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<
        W: std::io::Write,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > RulesDef<W, B, BR, P, Inner>
{
    pub fn add_rule(&mut self, rule: RuleDef<W, B, BR, P, Inner>) -> Result<(), Error> {
        match &rule {
            RuleDef::Filter(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Filter(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::Ignored(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Ignored(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::Map(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Map(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::WriteIgnored(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::WriteIgnored(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
        };
        self.rules.push(rule);
        Ok(())
    }

    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules
            .retain(|r| r.id().to_string() != rule.to_string());
    }

    pub fn ignore(&mut self, buffer: &[u8]) -> Result<(), Error> {
        for rule in self.rules.iter_mut() {
            match rule {
                RuleDef::Ignored(cb) => match cb {
                    RuleFnDef::Static(cb) => cb(buffer),
                    RuleFnDef::Dynamic(cb) => cb(buffer),
                },
                RuleDef::WriteIgnored(dest, cb) => match cb {
                    RuleFnDef::Static(cb) => {
                        cb(dest, buffer)?;
                    }
                    RuleFnDef::Dynamic(cb) => {
                        cb(dest, buffer)?;
                    }
                },
                _ignored => {}
            }
        }
        Ok(())
    }
    pub fn filter(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::Filter(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(referred),
            RuleFnDef::Dynamic(cb) => cb(referred),
        }
    }
    pub fn map(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> Result<(), Error> {
        let Some((writer, cb)) = self.rules.iter_mut().find_map(|r| {
            if let RuleDef::Map(writer, cb) = r {
                Some((writer, cb))
            } else {
                None
            }
        }) else {
            return Ok(());
        };
        match cb {
            RuleFnDef::Static(cb) => cb(writer, referred)?,
            RuleFnDef::Dynamic(cb) => cb(writer, referred)?,
        }
        Ok(())
    }
}

pub enum NextPacket<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    NotEnoughData(usize),
    NoData,
    NotFound,
    Ignored,
    Found(PacketDef<B, P, Inner>),
}

pub enum PacketHeaderState {
    NotFound,
    NotEnoughData(usize, usize),
    Found(PacketHeader, std::ops::RangeInclusive<usize>),
}

pub enum HeaderReadState {
    Ready(Option<PacketHeader>),
    Refill(Option<(Vec<u8>, usize)>),
    Empty,
}

pub enum ResolveHeaderReady<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> {
    Next(NextPacket<B, P, Inner>),
    Resolved(PacketHeader),
}

pub struct PacketBufReaderDef<
    'a,
    R: std::io::Read,
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    inner: std::io::BufReader<&'a mut R>,
    rules: RulesDef<W, B, BR, P, Inner>,
    recent: HeaderReadState,
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
    pub fn new(inner: &'a mut R) -> Self {
        Self {
            inner: std::io::BufReader::new(inner),
            rules: RulesDef::default(),
            recent: HeaderReadState::Empty,
            buffered: Vec::with_capacity(u32::MAX as usize),
        }
    }

    pub fn add_rule(&mut self, rule: RuleDef<W, B, BR, P, Inner>) -> Result<(), Error> {
        self.rules.add_rule(rule)
    }

    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules.remove_rule(rule);
    }

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
