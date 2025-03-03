use std::io::{BufRead, BufReader, Read};

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
    recent: Option<PacketHeader>,
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
            recent: None,
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
            println!(">>>>>>>>>>>>>>>>>>>>>>> 000006");
            return Ok(PacketHeaderState::NotEnoughData(offset, needed));
        }
        Ok(PacketHeaderState::Found(
            PacketHeader::read_from_slice(&buffer[offset..], false)?,
            std::ops::RangeInclusive::new(offset, offset + PacketHeader::ssize() as usize),
        ))
    }

    pub fn read(&mut self) -> Result<NextPacket<B, P, Inner>, Error> {
        let (mut reader, header, consume) = if let Some(header) = self.recent.take() {
            let buffer = self.inner.fill_buf()?;
            // Check do we have enough data to load packet
            let packet_size = header.size as usize;
            let available = self.buffered.len() + buffer.len();
            if packet_size > available {
                // Not enough data to load packet
                let consumed = buffer.len();
                self.buffered.extend_from_slice(buffer);
                self.inner.consume(consumed);
                self.recent = Some(header);
                println!(">>>>>>>>>>>>>>>>>>>>>>> 000001");
                return Ok(NextPacket::NotEnoughData(packet_size - available));
            }
            let rest_data = packet_size - self.buffered.len();
            println!(">>>>>>>>>>>>>>>>>>>>>>> 000007 with header: {header:?}; in buf: {}; rest (to be consumed): {rest_data}", self.buffered.len());
            // Copy and consume only needed data
            self.buffered.extend_from_slice(&buffer[..rest_data]);
            if buffer.len() >= rest_data + 2 {
                println!(
                    ">>>>>>>>>>>>>>>>>>>>>>> end on: {:?}",
                    &buffer[rest_data..rest_data + 2]
                );
            }
            self.inner.consume(rest_data);
            let reader = BufReader::new(self.buffered.as_slice());
            println!(
                ">>>>>>>>>>>>>>>>>>>>>>> 000007 in buf after: {};",
                self.buffered.len(),
            );
            (reader, header, None)
        } else {
            self.buffered.clear();
            let buffer = self.inner.fill_buf()?;
            if buffer.is_empty() {
                return Ok(NextPacket::NoData);
            }
            println!(">>>>>>>>>>>>>>>>>>>>>>> reading from {:?}", &buffer[..2]);
            let available = buffer.len();
            match PacketBufReaderDef::<'a, R, W, B, BR, P, Inner>::read_header(buffer)? {
                PacketHeaderState::NotFound => {
                    println!(
                        ">>>>>>>>>>>>>>>>>> NOT FOUND IN BUF: {} bytes",
                        buffer.len()
                    );
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
                        // Consume until valid signature
                        self.inner.consume(from - 1);
                    }
                    println!(">>>>>>>>>>>>>>>>>>>>>>> 000002");
                    return Ok(NextPacket::NotEnoughData(needed));
                }
                PacketHeaderState::Found(header, sgmt) => {
                    // PacketDef header has been found
                    if sgmt.start() > &0 {
                        self.rules.ignore(&buffer[..*sgmt.start()])?;
                    }
                    println!("HEADER READ: {header:?}: {sgmt:?}");
                    let packet_size = header.size as usize;
                    let needs = packet_size + *sgmt.end();
                    if needs > available {
                        println!(
                            ">>>>>>>>>>>>>>>>>>>>>>> before 000003, buffer = {}",
                            self.buffered.len()
                        );
                        // Not enough data to load packet
                        self.buffered.extend_from_slice(&buffer[*sgmt.end()..]);
                        self.inner.consume(available);
                        self.recent = Some(header);
                        println!(
                            ">>>>>>>>>>>>>>>>>>>>>>> 000003, buffer = {}",
                            self.buffered.len()
                        );
                        return Ok(NextPacket::NotEnoughData(needs - available));
                    }
                    // Consume header
                    self.inner.consume(*sgmt.end());
                    // Again fill buffer
                    let buffer = self.inner.fill_buf()?;
                    if buffer.len() < header.size as usize {
                        println!("**** IMPOSSIBLE **** ");
                        panic!("**** IMPOSSIBLE **** ");
                    }
                    println!(">>>>>>>>>>>>>>>>>>>>>>> all good, reading");
                    let consume = Some(header.size);
                    (
                        BufReader::new(&buffer[..header.size as usize]),
                        header,
                        consume,
                    )
                }
            }
        };
        let mut buffer = vec![0u8; header.blocks_len as usize];
        reader.read_exact(&mut buffer)?;
        let mut blocks = Vec::new();
        let mut processed = 0;
        let mut count = 0;
        if !buffer.is_empty() {
            loop {
                if count == MAX_BLOCKS_COUNT {
                    self.buffered.clear();
                    return Err(Error::MaxBlocksCount);
                }
                let blk = match BR::read_from_slice(&buffer[processed..], false) {
                    Ok(blk) => blk,
                    Err(err) => {
                        self.buffered.clear();
                        consume.map(|s| self.inner.consume(s as usize));
                        return Err(err);
                    }
                };
                if blk.size() == 0 {
                    self.buffered.clear();
                    consume.map(|s| self.inner.consume(s as usize));
                    return Err(Error::ZeroLengthBlock);
                }
                processed += blk.size() as usize;
                count += 1;
                blocks.push(blk);
                if processed == buffer.len() {
                    break;
                }
            }
        }
        println!(">>>>>>>>>>>>>>>>>>>>>>> blocks: {}", blocks.len());
        let referred = PacketReferred::new(blocks, header);
        self.rules.map(&referred)?;
        if !self.rules.filter(&referred) {
            // PacketDef marked as ignored
            self.buffered.clear();
            consume.map(|s| self.inner.consume(s as usize));
            return Ok(NextPacket::Ignored);
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
            println!(
                ">>>>>>>>>>>>>>>>>>>>>>> reading payload; needs: {}",
                header.size - header.blocks_len
            );
            let mut buffer = reader.fill_buf()?;
            println!(
                ">>>>>>>>>>>>>>>>>>>>>>> reading payload; has: {}",
                buffer.len()
            );
            match <PayloadHeader as TryReadFromBuffered>::try_read(&mut buffer) {
                Ok(ReadStatus::Success(header)) => {
                    reader.consume(header.size());
                    let mut buffer = reader.fill_buf()?;
                    println!(
                        ">>>>>>>>>>>>>>>>>>>>>>> payload header read: paylaod size: {} / {} / {}",
                        header.size(),
                        header.payload_len(),
                        buffer.len()
                    );
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(
                        &mut buffer,
                        &header,
                    )? {
                        ReadStatus::Success(payload) => {
                            reader.consume(header.payload_len());
                            pkg.payload = Some(payload);
                            println!(">>>>>>>>>>>>>>>>>>>>>>> PAYLOAD EXTRACTED");
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            // This is error, but not NextPacket::NotEnoughData because length of payload
                            // already has been check. If we are here - some data is invalid and
                            // it's an error
                            println!(">>>>>>>>>>>>>>>>>>>>>>> 000004");
                            self.buffered.clear();
                            consume.map(|s| self.inner.consume(s as usize));
                            return Err(Error::NotEnoughData(needed as usize));
                        }
                    }
                }
                Ok(ReadStatus::NotEnoughData(needed)) => {
                    // This is error, but not NextPacket::NotEnoughData because length of payload
                    // already has been check. If we are here - some data is invalid and
                    // it's an error
                    println!(">>>>>>>>>>>>>>>>>>>>>>> 000005");
                    self.buffered.clear();
                    consume.map(|s| self.inner.consume(s as usize));
                    return Err(Error::NotEnoughData(needed as usize));
                }
                Err(err) => {
                    self.buffered.clear();
                    consume.map(|s| self.inner.consume(s as usize));
                    return Err(err);
                }
            }
        }
        self.buffered.clear();
        consume.map(|s| self.inner.consume(s as usize));
        Ok(NextPacket::Found(pkg))
    }
}
