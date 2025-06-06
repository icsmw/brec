use brec::prelude::*;
use proptest::prelude::*;

use crate::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

brec::generate!();

#[derive(PartialEq, PartialOrd, Debug)]
struct WrappedPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

impl From<&WrappedPacket> for Packet {
    fn from(wrapped: &WrappedPacket) -> Self {
        Packet::new(wrapped.blocks.clone(), wrapped.payload.clone())
    }
}

impl From<Packet> for WrappedPacket {
    fn from(pkg: Packet) -> Self {
        WrappedPacket {
            blocks: pkg.blocks,
            payload: pkg.payload,
        }
    }
}

impl Arbitrary for WrappedPacket {
    type Parameters = bool;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(no_blocks: bool) -> Self::Strategy {
        if no_blocks {
            prop::option::of(Payload::arbitrary())
                .prop_map(|payload| WrappedPacket {
                    blocks: Vec::new(),
                    payload,
                })
                .boxed()
        } else {
            (
                prop::collection::vec(Block::arbitrary(), 1..20),
                prop::option::of(Payload::arbitrary()),
            )
                .prop_map(|(blocks, payload)| WrappedPacket { blocks, payload })
                .boxed()
        }
    }
}

#[derive(Debug)]
struct LitteredPacket {
    pub packet: WrappedPacket,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

impl Arbitrary for LitteredPacket {
    type Parameters = bool;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(no_blocks: bool) -> Self::Strategy {
        (
            WrappedPacket::arbitrary_with(no_blocks),
            prop::option::of(prop::collection::vec(any::<u8>(), 1..2500)),
            prop::option::of(prop::collection::vec(any::<u8>(), 1..2500)),
        )
            .prop_map(|(packet, before, after)| LitteredPacket {
                packet,
                before,
                after,
            })
            .boxed()
    }
}

fn write_to_buf<W: std::io::Write>(
    buffer: &mut W,
    packets: &[WrappedPacket],
) -> std::io::Result<()> {
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(buffer)?;
    }
    Ok(())
}

fn write_to_buf_with_litter<W: std::io::Write>(
    buffer: &mut W,
    packets: &[LitteredPacket],
) -> std::io::Result<usize> {
    let mut litter_len = 0;
    for wrapped in packets.iter() {
        let mut packet: Packet = (&wrapped.packet).into();
        if let Some(litter) = wrapped.before.as_ref() {
            litter_len += litter.len();
            buffer.write_all(litter)?;
        }
        packet.write_all(buffer)?;
        if let Some(litter) = wrapped.after.as_ref() {
            litter_len += litter.len();
            buffer.write_all(litter)?;
        }
    }
    Ok(litter_len)
}

fn read_packets(buffer: &[u8]) -> std::io::Result<(usize, Vec<Packet>)> {
    use std::io::{BufReader, Cursor};

    let mut packets: Vec<Packet> = Vec::new();
    let mut inner = BufReader::new(Cursor::new(buffer));
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut inner);
    let litter_len: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let litter_len_inner = litter_len.clone();
    reader
        .add_rule(Rule::Ignored(brec::RuleFnDef::Dynamic(Box::new(
            move |bytes: &[u8]| {
                litter_len_inner.fetch_add(bytes.len(), Ordering::SeqCst);
            },
        ))))
        .unwrap();
    let prefilter_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let prefilter_count_inner = prefilter_count.clone();
    reader
        .add_rule(Rule::FilterByBlocks(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                prefilter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    let payload_filter_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let payload_filter_count_inner = payload_filter_count.clone();
    reader
        .add_rule(Rule::FilterByPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                payload_filter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    let filter_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let filter_count_inner = filter_count.clone();
    reader
        .add_rule(Rule::Filter(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                filter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    loop {
        match reader.read() {
            Ok(next) => match next {
                NextPacket::Found(packet) => packets.push(packet),
                NextPacket::NotFound => {
                    // Data will be refilled with next call
                }
                NextPacket::NotEnoughData(_needed) => {
                    // Data will be refilled with next call
                }
                NextPacket::NoData => {
                    break;
                }
                NextPacket::Skipped => {
                    //
                }
            },
            Err(err) => {
                println!("ERR: {err}");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err.to_string(),
                ));
            }
        };
    }
    assert_eq!(packets.len(), prefilter_count.load(Ordering::SeqCst));
    assert_eq!(packets.len(), filter_count.load(Ordering::SeqCst));
    assert_eq!(
        packets.iter().filter(|pkg| pkg.payload.is_some()).count(),
        payload_filter_count.load(Ordering::SeqCst)
    );
    Ok((litter_len.load(Ordering::SeqCst), packets))
}

fn read_packets_one_by_one(bytes: &[Vec<u8>]) -> Result<Vec<WrappedPacket>, brec::Error> {
    let mut packets = Vec::new();
    for inner in bytes.iter() {
        let a = match <Packet as ReadFrom>::read(&mut std::io::Cursor::new(inner)) {
            Ok(res) => res,
            Err(err) => {
                println!("Err: {err}");
                return Err(err);
            }
        };
        let b = match <Packet as TryReadFrom>::try_read(&mut std::io::Cursor::new(inner)) {
            Ok(res) => res,
            Err(err) => {
                println!("Err: {err}");
                return Err(err);
            }
        };
        let c = match <Packet as TryReadFromBuffered>::try_read(&mut std::io::Cursor::new(inner)) {
            Ok(res) => res,
            Err(err) => {
                println!("Err: {err}");
                return Err(err);
            }
        };
        if let (ReadStatus::Success(b), ReadStatus::Success(c)) = (b, c) {
            let a = Into::<WrappedPacket>::into(a);
            assert_eq!(a, Into::<WrappedPacket>::into(b));
            assert_eq!(a, Into::<WrappedPacket>::into(c));
            packets.push(a);
        }
    }
    Ok(packets)
}

fn read_packets_with_read_from(inner: &[u8]) -> Result<Vec<WrappedPacket>, brec::Error> {
    let mut packets = Vec::new();
    let mut cursor = std::io::Cursor::new(inner);
    loop {
        let pkg = match <Packet as ReadFrom>::read(&mut cursor) {
            Ok(pkg) => Into::<WrappedPacket>::into(pkg),
            Err(err) => {
                println!("Err: {err}");
                break;
            }
        };
        packets.push(pkg);
    }
    Ok(packets)
}

fn read_packets_with_try_read_from(inner: &[u8]) -> Result<Vec<WrappedPacket>, brec::Error> {
    let mut packets = Vec::new();
    let mut cursor = std::io::Cursor::new(inner);
    loop {
        let pkg = match <Packet as TryReadFrom>::try_read(&mut cursor) {
            Ok(ReadStatus::Success(pkg)) => Into::<WrappedPacket>::into(pkg),
            Ok(ReadStatus::NotEnoughData(_needed)) => {
                break;
            }
            Err(err) => {
                println!("Err: {err}");
                break;
            }
        };
        packets.push(pkg);
    }
    Ok(packets)
}

fn read_packets_with_try_read_from_buffered(
    inner: &[u8],
) -> Result<Vec<WrappedPacket>, brec::Error> {
    let mut packets = Vec::new();
    let mut cursor = std::io::Cursor::new(inner);
    loop {
        let pkg = match <Packet as TryReadFromBuffered>::try_read(&mut cursor) {
            Ok(ReadStatus::Success(pkg)) => Into::<WrappedPacket>::into(pkg),
            Ok(ReadStatus::NotEnoughData(_needed)) => {
                break;
            }
            Err(err) => {
                println!("Err: {err}");
                break;
            }
        };
        packets.push(pkg);
    }
    Ok(packets)
}

static BYTES: AtomicUsize = AtomicUsize::new(0);
static INSTANCES: AtomicUsize = AtomicUsize::new(0);

fn report(bytes: usize, instance: usize) {
    use num_format::{Locale, ToFormattedString};

    BYTES.fetch_add(bytes, Ordering::Relaxed);
    INSTANCES.fetch_add(instance, Ordering::Relaxed);
    let bytes = BYTES.load(Ordering::Relaxed);
    println!(
        "Generated {} packets ({}, {} B)",
        INSTANCES
            .load(Ordering::Relaxed)
            .to_formatted_string(&Locale::en),
        if bytes > 1024 * 1024 {
            format!(
                "{} Mb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else if bytes > 1024 {
            format!(
                "{} Kb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else {
            format!("{} B", bytes.to_formatted_string(&Locale::en))
        },
        bytes.to_formatted_string(&Locale::en)
    );
}

static STORED_PACKETS: AtomicUsize = AtomicUsize::new(0);
static BLOCKS_VISITED: AtomicUsize = AtomicUsize::new(0);

fn report_storage(packets: usize, blocks_visited: Option<usize>) {
    use num_format::{Locale, ToFormattedString};
    STORED_PACKETS.fetch_add(packets, Ordering::Relaxed);
    if let Some(visited) = blocks_visited {
        BLOCKS_VISITED.fetch_add(visited, Ordering::Relaxed);
        println!(
            "Generated, stored and read {} packets; blocks visited: {}",
            STORED_PACKETS
                .load(Ordering::Relaxed)
                .to_formatted_string(&Locale::en),
            BLOCKS_VISITED
                .load(Ordering::Relaxed)
                .to_formatted_string(&Locale::en),
        );
    } else {
        println!(
            "Generated, stored and read {} packets;",
            STORED_PACKETS
                .load(Ordering::Relaxed)
                .to_formatted_string(&Locale::en),
        );
    }
}

fn try_read_from(packets: Vec<WrappedPacket>) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    write_to_buf(&mut buf, &packets)?;
    let (_, restored) = read_packets(&buf)?;
    let count = restored.len();
    assert_eq!(packets.len(), count);
    for (left, right) in restored
        .into_iter()
        .map(|pkg| pkg.into())
        .collect::<Vec<WrappedPacket>>()
        .iter()
        .zip(packets.iter())
    {
        assert_eq!(left, right);
    }
    report(buf.len(), count);
    Ok(())
}

fn try_read_with_litter(packets: Vec<LitteredPacket>) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let litter_len = write_to_buf_with_litter(&mut buf, &packets)?;
    let (read_litter_len, restored) = read_packets(&buf)?;
    assert_eq!(litter_len, read_litter_len);
    let count = restored.len();
    let packets = packets
        .into_iter()
        .map(|p| p.packet)
        .collect::<Vec<WrappedPacket>>();
    assert_eq!(packets.len(), count);
    for (left, right) in restored
        .into_iter()
        .map(|pkg| pkg.into())
        .collect::<Vec<WrappedPacket>>()
        .iter()
        .zip(packets.iter())
    {
        assert_eq!(left, right);
    }
    report(buf.len(), count);
    Ok(())
}

fn try_reading_one_by_one(packets: Vec<WrappedPacket>) -> std::io::Result<()> {
    let mut bufs = Vec::new();
    let mut bytes = 0;
    for wrapped in packets.iter() {
        let mut buf: Vec<u8> = Vec::new();
        let mut packet: Packet = wrapped.into();
        packet.write_all(&mut buf)?;
        bytes += buf.len();
        bufs.push(buf);
    }
    let restored = read_packets_one_by_one(&bufs)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored.iter().zip(packets.iter()) {
        assert_eq!(left, right);
    }
    report(bytes, packets.len());
    Ok(())
}

fn try_reading_with_read(packets: Vec<WrappedPacket>) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(&mut buffer)?;
    }
    let restored = read_packets_with_read_from(&buffer)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored.iter().zip(packets.iter()) {
        assert_eq!(left, right);
    }
    report(buffer.len(), packets.len());
    Ok(())
}

fn try_reading_with_try_read(packets: Vec<WrappedPacket>) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(&mut buffer)?;
    }
    let restored = read_packets_with_try_read_from(&buffer)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored.iter().zip(packets.iter()) {
        assert_eq!(left, right);
    }
    report(buffer.len(), packets.len());
    Ok(())
}

fn try_reading_with_try_read_buffered(packets: Vec<WrappedPacket>) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(&mut buffer)?;
    }
    let restored = read_packets_with_try_read_from_buffered(&buffer)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored.iter().zip(packets.iter()) {
        assert_eq!(left, right);
    }
    report(buffer.len(), packets.len());
    Ok(())
}

fn storage_write_read_filter(packets: Vec<WrappedPacket>, filename: &str) -> std::io::Result<()> {
    if packets.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            String::from("Empty packets; no packets to test"),
        ));
    }
    let tmp = std::env::temp_dir().join(filename);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut storage = Storage::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    for packet in packets.iter() {
        storage
            .insert(packet.into())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    let storage_count = storage.count();
    assert_eq!(packets.len(), storage_count);
    let mut restored = Vec::new();
    for packet in storage.iter() {
        match packet {
            Ok(packet) => {
                restored.push(packet);
            }
            Err(err) => {
                panic!("Fail to read storage: {err}");
            }
        }
    }
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored
        .into_iter()
        .map(|pkg| pkg.into())
        .collect::<Vec<WrappedPacket>>()
        .iter()
        .zip(packets.iter())
    {
        assert_eq!(left, right);
    }
    let blocks_visited: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let blocks_visited_inner = blocks_visited.clone();
    storage
        .add_rule(Rule::FilterByBlocks(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks: &[BlockReferred]| {
                blocks_visited_inner.fetch_add(blocks.len(), Ordering::SeqCst);
                false
            },
        ))))
        .unwrap();
    // Read each 2th and 3th packets
    for n in 0..packets.len() {
        if n % 2 != 0 && n % 3 != 0 {
            continue;
        }
        // Test nth packet reading
        if let Some(packet) = storage
            .nth(n)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?
        {
            assert_eq!(Into::<WrappedPacket>::into(packet), packets[n]);
        }
        // Test range reading
        if n + 10 < packets.len() - 1 {
            for (i, packet) in storage.range(n, 10).enumerate() {
                assert_eq!(
                    Into::<WrappedPacket>::into(packet.map_err(|err| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        err.to_string()
                    ))?),
                    packets[n + i]
                );
            }
            for (i, packet) in storage.range_filtered(n, n + 10).enumerate() {
                assert_eq!(
                    Into::<WrappedPacket>::into(packet.map_err(|err| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        err.to_string()
                    ))?),
                    packets[n + i]
                );
            }
        }
    }
    let payload_visited: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let payload_visited_inner = payload_visited.clone();
    storage.remove_rule(RuleDefId::FilterByBlocks);
    storage
        .add_rule(Rule::FilterByPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |_payload: &[u8]| {
                payload_visited_inner.fetch_add(1, Ordering::SeqCst);
                false
            },
        ))))
        .unwrap();
    for _ in storage.filtered() {
        // Itarate all
    }
    let payloads = packets.iter().filter(|pkg| pkg.payload.is_some()).count();
    assert_eq!(payloads, payload_visited.load(Ordering::SeqCst));
    // Create new storage to same file
    let mut storage = Storage::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    // Add new packet
    storage
        .insert((&packets[0]).into())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(storage.count(), storage_count + 1);
    report_storage(packets.len(), Some(blocks_visited.load(Ordering::SeqCst)));
    Ok(())
}

fn storage_slot_locator(
    packet: &WrappedPacket,
    filename: &str,
    count: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut storage = Storage::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    for _ in 0..count {
        storage
            .insert(packet.into())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    assert_eq!(count, storage.count());
    // Locator should setup it self to correct position
    let mut storage = Storage::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(count, storage.count());
    storage
        .insert(packet.into())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(count + 1, storage.count());
    Ok(())
}

fn get_proptest_config() -> ProptestConfig {
    let cases = std::env::var("BREC_STRESS_PACKETS_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    ProptestConfig {
        max_shrink_iters: 50,
        max_local_rejects: 1_000_000,
        ..ProptestConfig::with_cases(cases)
    }
}

fn max() -> usize {
    std::env::var("BREC_STRESS_PACKETS_MAX_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

const STORAGE_WRITE_READ_FILTER_NO_BLOCKS_FILE: &str =
    "test_storage_write_read_filter_no_blocks.bin";
const STORAGE_WRITE_READ_FILTER_BLOCKS_FILE: &str =
    "test_storage_write_read_filter_with_blocks.bin";
const STORAGE_SLOT_LOCATOR_FILE: &str = "test_storage_slot_locator.bin";

proptest! {
    #![proptest_config(get_proptest_config())]

    #[test]
    fn try_read_from_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        try_read_from(packets)?;
    }

    #[test]
    fn try_read_from_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        try_read_from(packets)?;
    }

    #[test]
    fn try_read_with_litter_no_blocks(packets in proptest::collection::vec(LitteredPacket::arbitrary_with(true), 1..max())) {
        try_read_with_litter(packets)?;
    }

    #[test]
    fn try_read_with_litter_with_blocks(packets in proptest::collection::vec(LitteredPacket::arbitrary_with(false), 1..max())) {
        try_read_with_litter(packets)?;
    }

    #[test]
    fn try_reading_one_by_one_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        try_reading_one_by_one(packets)?;
    }

    #[test]
    fn try_reading_one_by_one_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        try_reading_one_by_one(packets)?;
    }

    #[test]
    fn try_reading_with_read_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        try_reading_with_read(packets)?;
    }

    #[test]
    fn try_reading_with_read_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        try_reading_with_read(packets)?;
    }

    #[test]
    fn try_reading_with_try_read_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        try_reading_with_try_read(packets)?;
    }

    #[test]
    fn try_reading_with_try_read_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        try_reading_with_try_read(packets)?;
    }

    #[test]
    fn try_reading_with_try_read_buffered_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        try_reading_with_try_read_buffered(packets)?;
    }

    #[test]
    fn try_reading_with_try_read_buffered_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        try_reading_with_try_read_buffered(packets)?;
    }

    #[test]
    fn storage_write_read_filter_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        storage_write_read_filter(packets, STORAGE_WRITE_READ_FILTER_NO_BLOCKS_FILE)?;
    }

    #[test]
    fn storage_write_read_filter_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        storage_write_read_filter(packets, STORAGE_WRITE_READ_FILTER_BLOCKS_FILE)?;
    }

    #[test]
    fn storage_slot_locator_test(packet in WrappedPacket::arbitrary_with(false)) {
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, 0)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, 1)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY - 1)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY + 2)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY * 2)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY * 2 - 1)?;
        storage_slot_locator(&packet, STORAGE_SLOT_LOCATOR_FILE, DEFAULT_SLOT_CAPACITY * 2 + 1)?;
    }

}
