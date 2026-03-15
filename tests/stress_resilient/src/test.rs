use brec::prelude::*;
use proptest::prelude::*;

use crate::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
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
        packet.write_all(buffer, &mut ())?;
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
        packet.write_all(buffer, &mut ())?;
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
        .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                prefilter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    let payload_filter_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let payload_filter_count_inner = payload_filter_count.clone();
    reader
        .add_rule(Rule::FilterPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                payload_filter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    let filter_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let filter_count_inner = filter_count.clone();
    reader
        .add_rule(Rule::FilterPacket(brec::RuleFnDef::Dynamic(Box::new(
            move |_| {
                filter_count_inner.fetch_add(1, Ordering::SeqCst);
                true
            },
        ))))
        .unwrap();
    loop {
        match reader.read(&mut ()) {
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
                eprintln!("ERR: {err}");
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
        let a = match <Packet as ReadPacketFrom>::read(&mut std::io::Cursor::new(inner), &mut ()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Err: {err}");
                return Err(err);
            }
        };
        let b = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(inner), &mut ()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Err: {err}");
                return Err(err);
            }
        };
        let c = match <Packet as TryReadPacketFromBuffered>::try_read(&mut std::io::Cursor::new(inner), &mut ()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Err: {err}");
                return Err(err);
            }
        };
        if let (
            PacketReadStatus::Success((b, _)),
            PacketReadStatus::Success((c, _)),
        ) = (b, c)
        {
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
        let pkg = match <Packet as ReadPacketFrom>::read(&mut cursor, &mut ()) {
            Ok(pkg) => Into::<WrappedPacket>::into(pkg),
            Err(err) => {
                println!("Err (expected): {err}");
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
        let pkg = match <Packet as TryReadPacketFrom>::try_read(&mut cursor, &mut ()) {
            Ok(PacketReadStatus::Success((pkg, _))) => Into::<WrappedPacket>::into(pkg),
            Ok(PacketReadStatus::NotEnoughData(_needed)) => {
                break;
            }
            Err(err) => {
                println!("Err (expected): {err}");
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
        let pkg = match <Packet as TryReadPacketFromBuffered>::try_read(&mut cursor, &mut ()) {
            Ok(PacketReadStatus::Success((pkg, _))) => Into::<WrappedPacket>::into(pkg),
            Ok(PacketReadStatus::NotEnoughData(_needed)) => {
                break;
            }
            Err(err) => {
                println!("Err (expected): {err}");
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
        packet.write_all(&mut buf, &mut ())?;
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
        packet.write_all(&mut buffer, &mut ())?;
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
        packet.write_all(&mut buffer, &mut ())?;
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
        packet.write_all(&mut buffer, &mut ())?;
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
    let mut writer = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    for packet in packets.iter() {
        writer
            .insert(packet.into(), &mut ())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    let mut reader = Reader::new(&file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let storage_count = reader.count();
    assert_eq!(packets.len(), storage_count);
    let mut restored = Vec::new();
    for packet in reader.iter(&mut ()) {
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
    reader
        .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks| {
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
        if let Some(packet) = reader
            .nth(n, &mut ())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?
        {
            assert_eq!(Into::<WrappedPacket>::into(packet), packets[n]);
        }
        // Test range reading
        if n + 10 < packets.len() - 1 {
            for (i, packet) in reader.range(n, 10, &mut ()).enumerate() {
                assert_eq!(
                    Into::<WrappedPacket>::into(packet.map_err(|err| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        err.to_string()
                    ))?),
                    packets[n + i]
                );
            }
            for (i, packet) in reader.range_filtered(n, n + 10, &mut ()).enumerate() {
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
    reader.remove_rule(RuleDefId::Prefilter);
    reader
        .add_rule(Rule::FilterPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |_payload: &[u8]| {
                payload_visited_inner.fetch_add(1, Ordering::SeqCst);
                false
            },
        ))))
        .unwrap();
    for _ in reader.filtered(&mut ()) {
        // Itarate all
    }
    let payloads = packets.iter().filter(|pkg| pkg.payload.is_some()).count();
    assert_eq!(payloads, payload_visited.load(Ordering::SeqCst));
    // Create new storage to same file
    let mut writer = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    // Add new packet
    writer
        .insert((&packets[0]).into(), &mut ())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let reader = Reader::new(&file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(reader.count(), storage_count + 1);
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
    let mut writer = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    for _ in 0..count {
        writer
            .insert(packet.into(), &mut ())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    let reader = Reader::new(&file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(count, reader.count());
    // Locator should setup it self to correct position
    let mut writer = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    writer
        .insert(packet.into(), &mut ())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let reader = Reader::new(&file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    assert_eq!(count + 1, reader.count());
    Ok(())
}

fn storage_write_stream_read(packets: Vec<WrappedPacket>, filename: &str) -> std::io::Result<()> {
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

    let mut writer = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    for packet in packets.iter() {
        writer
            .insert(packet.into(), &mut ())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    drop(writer);
    file.sync_all()?;

    let clean = std::fs::read(&tmp)?;
    let (clean_litter_len, clean_restored) = read_packets(&clean)?;
    assert_eq!(packets.len(), clean_restored.len());
    for (left, right) in clean_restored
        .into_iter()
        .map(Into::<WrappedPacket>::into)
        .collect::<Vec<WrappedPacket>>()
        .iter()
        .zip(packets.iter())
    {
        assert_eq!(left, right);
    }

    // Storage file already contains non-packet service data (slots/CRC/hash), which
    // PacketBufReader treats as ignored bytes. Add extra noise around the file to ensure
    // packet detection is still stable in a "dirty" stream.
    let before = vec![0xAA; 97];
    let after = vec![0x55; 73];
    let mut littered = Vec::with_capacity(before.len() + clean.len() + after.len());
    littered.extend_from_slice(&before);
    littered.extend_from_slice(&clean);
    littered.extend_from_slice(&after);

    let (litter_len, restored) = read_packets(&littered)?;
    assert_eq!(clean_litter_len + before.len() + after.len(), litter_len);
    assert_eq!(packets.len(), restored.len());
    for (left, right) in restored
        .into_iter()
        .map(Into::<WrappedPacket>::into)
        .collect::<Vec<WrappedPacket>>()
        .iter()
        .zip(packets.iter())
    {
        assert_eq!(left, right);
    }
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
const STORAGE_WRITE_STREAM_READ_NO_BLOCKS_FILE: &str =
    "test_storage_write_stream_read_no_blocks.bin";
const STORAGE_WRITE_STREAM_READ_BLOCKS_FILE: &str =
    "test_storage_write_stream_read_with_blocks.bin";

#[cfg(feature = "resilient")]
fn packet_bytes_with(blocks: Vec<Block>, payload: Option<Payload>) -> Vec<u8> {
    let wrapped = WrappedPacket { blocks, payload };
    let mut bytes = Vec::new();
    write_to_buf(&mut bytes, &[wrapped]).unwrap();
    bytes
}

#[cfg(feature = "resilient")]
fn blocks_offset() -> usize {
    PacketHeader::SIZE as usize
}

#[cfg(feature = "resilient")]
fn packet_blocks_len(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes[16..24].try_into().unwrap())
}

#[cfg(feature = "resilient")]
fn block_total_len(bytes: &[u8], offset: usize) -> usize {
    let body_len = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
    4 + 4 + body_len + 4
}

#[cfg(feature = "resilient")]
fn payload_offsets(bytes: &[u8]) -> (usize, usize, usize, usize, usize, u32) {
    let payload_off = blocks_offset() + packet_blocks_len(bytes) as usize;
    let sig_len = bytes[payload_off] as usize;
    let crc_len_pos = payload_off + 1 + sig_len;
    let crc_len = bytes[crc_len_pos] as usize;
    let payload_len_pos = crc_len_pos + 1 + crc_len;
    let payload_body_off = payload_len_pos + 4;
    let payload_len =
        u32::from_le_bytes(bytes[payload_len_pos..payload_len_pos + 4].try_into().unwrap());
    (
        payload_off,
        sig_len,
        crc_len_pos,
        payload_len_pos,
        payload_body_off,
        payload_len,
    )
}

#[cfg(feature = "resilient")]
fn unwrap_packet_success(
    status: PacketReadStatus<Packet>,
) -> (Packet, Vec<brec::Unrecognized>) {
    match status {
        PacketReadStatus::Success((packet, skipped)) => (packet, skipped),
        PacketReadStatus::NotEnoughData(n) => panic!("unexpected NotEnoughData: {n}"),
    }
}

#[cfg(feature = "resilient")]
fn unknown_block_signature(block_bytes: &[u8]) -> [u8; 4] {
    let current_sig: [u8; 4] = block_bytes[..4].try_into().unwrap();
    for i in 1u8..=u8::MAX {
        let mut candidate = current_sig;
        candidate[0] ^= i;
        let mut probe = block_bytes.to_vec();
        probe[..4].copy_from_slice(&candidate);
        let verdict = <Block as TryReadFrom>::try_read(&mut std::io::Cursor::new(&probe));
        if matches!(verdict, Err(brec::Error::SignatureDismatch(_))) {
            return candidate;
        }
    }
    panic!("cannot find unknown block signature candidate");
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_in_middle() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let b3 = Block::BlockEnums(BlockEnums {
        level: Level::Info,
        kind: Kind::Socket,
    });
    let mut bytes = packet_bytes_with(vec![b1.clone(), b2.clone(), b3.clone()], None);
    let block0_off = blocks_offset();
    let block1_off = block0_off + block_total_len(&bytes, block0_off);
    let block1_len = block_total_len(&bytes, block1_off);
    let unknown_sig = unknown_block_signature(&bytes[block1_off..block1_off + block1_len]);
    bytes[block1_off..block1_off + 4].copy_from_slice(&unknown_sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b1, b3]);
    assert_eq!(packet.payload, None);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(
        skipped[0].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == unknown_sig
    ));
    assert_eq!(skipped[0].pos, Some(block1_off as u64));
    assert_eq!(
        skipped[0].len,
        Some(u32::from_le_bytes(bytes[block1_off + 4..block1_off + 8].try_into().unwrap()) as u64)
    );
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_multiple_unknown_blocks() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let b3 = Block::BlockEnums(BlockEnums {
        level: Level::Info,
        kind: Kind::Socket,
    });
    let mut bytes = packet_bytes_with(vec![b1, b2, b3.clone()], None);
    let off0 = blocks_offset();
    let off1 = off0 + block_total_len(&bytes, off0);
    let len0 = block_total_len(&bytes, off0);
    let len1 = block_total_len(&bytes, off1);
    let sig0 = unknown_block_signature(&bytes[off0..off0 + len0]);
    let sig1 = unknown_block_signature(&bytes[off1..off1 + len1]);
    bytes[off0..off0 + 4].copy_from_slice(&sig0);
    bytes[off1..off1 + 4].copy_from_slice(&sig1);

    let status = <Packet as TryReadPacketFromBuffered>::try_read(
        &mut std::io::Cursor::new(&bytes),
        &mut (),
    )
    .expect("buffered read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b3]);
    assert_eq!(skipped.len(), 2);
    assert!(matches!(
        skipped[0].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == sig0
    ));
    assert!(matches!(
        skipped[1].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == sig1
    ));
    assert_eq!(skipped[0].pos, Some(off0 as u64));
    assert_eq!(skipped[1].pos, Some(off1 as u64));
    assert_eq!(
        skipped[0].len,
        Some(u32::from_le_bytes(bytes[off0 + 4..off0 + 8].try_into().unwrap()) as u64)
    );
    assert_eq!(
        skipped[1].len,
        Some(u32::from_le_bytes(bytes[off1 + 4..off1 + 8].try_into().unwrap()) as u64)
    );
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_payload_is_skipped() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Debug,
        kind: Kind::Socket,
    });
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![block.clone()], Some(payload));
    let (payload_off, sig_len, _, _, _, payload_len) = payload_offsets(&bytes);
    let unknown_sig = vec![0xAB; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![block]);
    assert_eq!(packet.payload, None);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(
        &skipped[0].sig,
        brec::UnrecognizedSignature::Payload(sig) if sig == &unknown_sig
    ));
    assert_eq!(skipped[0].pos, Some((payload_off + 1) as u64));
    assert_eq!(skipped[0].len, Some(payload_len as u64));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_first() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let mut bytes = packet_bytes_with(vec![b1.clone(), b2.clone()], None);
    let first_off = blocks_offset();
    let first_len = block_total_len(&bytes, first_off);
    let unknown_sig = unknown_block_signature(&bytes[first_off..first_off + first_len]);
    bytes[first_off..first_off + 4].copy_from_slice(&unknown_sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b2]);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(
        skipped[0].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == unknown_sig
    ));
    assert_eq!(skipped[0].pos, Some(first_off as u64));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_last() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let mut bytes = packet_bytes_with(vec![b1.clone(), b2.clone()], None);
    let first_off = blocks_offset();
    let last_off = first_off + block_total_len(&bytes, first_off);
    let last_len = block_total_len(&bytes, last_off);
    let unknown_sig = unknown_block_signature(&bytes[last_off..last_off + last_len]);
    bytes[last_off..last_off + 4].copy_from_slice(&unknown_sig);

    let status = <Packet as TryReadPacketFromBuffered>::try_read(
        &mut std::io::Cursor::new(&bytes),
        &mut (),
    )
    .expect("buffered read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b1]);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(
        skipped[0].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == unknown_sig
    ));
    assert_eq!(skipped[0].pos, Some(last_off as u64));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_payload_without_blocks_is_skipped() {
    let payload = Payload::PayloadD(PayloadD::U32(7));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (payload_off, sig_len, _, _, _, payload_len) = payload_offsets(&bytes);
    let unknown_sig = vec![0xCD; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert!(packet.blocks.is_empty());
    assert_eq!(packet.payload, None);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(
        &skipped[0].sig,
        brec::UnrecognizedSignature::Payload(sig) if sig == &unknown_sig
    ));
    assert_eq!(skipped[0].pos, Some((payload_off + 1) as u64));
    assert_eq!(skipped[0].len, Some(payload_len as u64));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_multiple_unknown_blocks_with_known_payload() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let b3 = Block::BlockEnums(BlockEnums {
        level: Level::Info,
        kind: Kind::Socket,
    });
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![b1, b2, b3.clone()], Some(payload.clone()));
    let off0 = blocks_offset();
    let off1 = off0 + block_total_len(&bytes, off0);
    let len0 = block_total_len(&bytes, off0);
    let len1 = block_total_len(&bytes, off1);
    let sig0 = unknown_block_signature(&bytes[off0..off0 + len0]);
    let sig1 = unknown_block_signature(&bytes[off1..off1 + len1]);
    bytes[off0..off0 + 4].copy_from_slice(&sig0);
    bytes[off1..off1 + 4].copy_from_slice(&sig1);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b3]);
    assert_eq!(packet.payload, Some(payload));
    assert_eq!(skipped.len(), 2);
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_and_unknown_payload_are_both_skipped() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![b1.clone(), b2.clone()], Some(payload));
    let block_off = blocks_offset();
    let block_len = block_total_len(&bytes, block_off);
    let block_sig = unknown_block_signature(&bytes[block_off..block_off + block_len]);
    bytes[block_off..block_off + 4].copy_from_slice(&block_sig);
    let (payload_off, sig_len, _, _, _, payload_len) = payload_offsets(&bytes);
    let payload_sig = vec![0xEF; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&payload_sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert_eq!(packet.blocks, vec![b2]);
    assert_eq!(packet.payload, None);
    assert_eq!(skipped.len(), 2);
    assert!(matches!(
        skipped[0].sig,
        brec::UnrecognizedSignature::Block(sig) if sig == block_sig
    ));
    assert!(matches!(
        &skipped[1].sig,
        brec::UnrecognizedSignature::Payload(sig) if sig == &payload_sig
    ));
    assert_eq!(skipped[0].pos, Some(block_off as u64));
    assert_eq!(skipped[1].pos, Some((payload_off + 1) as u64));
    assert_eq!(skipped[1].len, Some(payload_len as u64));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_known_block_crc_error_is_hard_error() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let crc_off = off + block_total_len(&bytes, off) - 4;
    bytes[crc_off] ^= 0xFF;

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail on corrupted known block crc"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::CrcDismatch));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_known_payload_crc_error_is_hard_error() {
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (_, _, _, _, payload_body_off, _) = payload_offsets(&bytes);
    bytes[payload_body_off] ^= 0xFF;

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail on corrupted known payload body"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::CrcDismatch));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_zero_len_returns_invalid_length() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let len = block_total_len(&bytes, off);
    let sig = unknown_block_signature(&bytes[off..off + len]);
    bytes[off..off + 4].copy_from_slice(&sig);
    bytes[off + 4..off + 8].copy_from_slice(&0u32.to_le_bytes());

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail on zero unknown block len"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::InvalidLength));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_payload_len_overflow_returns_invalid_length() {
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (payload_off, sig_len, _, payload_len_pos, _, _) = payload_offsets(&bytes);
    let unknown_sig = vec![0xAA; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);
    bytes[payload_len_pos..payload_len_pos + 4].copy_from_slice(&u32::MAX.to_le_bytes());

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail on unknown payload len overflow"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::InvalidLength));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_partial_unknown_block_header_returns_not_enough_data() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let len = block_total_len(&bytes, off);
    let sig = unknown_block_signature(&bytes[off..off + len]);
    bytes[off..off + 4].copy_from_slice(&sig);
    let truncated = bytes[..off + 4 + 2].to_vec();

    let status =
        <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&truncated), &mut ())
            .expect("partial packet must not hard fail");
    assert!(matches!(status, PacketReadStatus::NotEnoughData(_)));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_partial_unknown_block_header_buffered_returns_not_enough_data() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let len = block_total_len(&bytes, off);
    let sig = unknown_block_signature(&bytes[off..off + len]);
    bytes[off..off + 4].copy_from_slice(&sig);
    let truncated = bytes[..off + 4 + 2].to_vec();

    let status = <Packet as TryReadPacketFromBuffered>::try_read(
        &mut std::io::Cursor::new(&truncated),
        &mut (),
    )
    .expect("partial packet must not hard fail");
    assert!(matches!(status, PacketReadStatus::NotEnoughData(_)));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_partial_unknown_payload_header_returns_not_enough_data() {
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (payload_off, sig_len, _, _, _, _) = payload_offsets(&bytes);
    let unknown_sig = vec![0xAA; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);
    let truncated = bytes[..payload_off + 1 + sig_len + 1].to_vec();

    let status =
        <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&truncated), &mut ())
            .expect("partial packet must not hard fail");
    assert!(matches!(status, PacketReadStatus::NotEnoughData(_)));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_partial_unknown_payload_body_returns_not_enough_data() {
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (payload_off, sig_len, _, _, payload_body_off, payload_len) = payload_offsets(&bytes);
    let unknown_sig = vec![0xAB; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);
    let truncated = bytes[..payload_body_off + payload_len as usize - 1].to_vec();

    let status =
        <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&truncated), &mut ())
            .expect("partial packet must not hard fail");
    assert!(matches!(status, PacketReadStatus::NotEnoughData(_)));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_partial_unknown_payload_body_buffered_returns_not_enough_data() {
    let payload = Payload::PayloadD(PayloadD::U32(42));
    let mut bytes = packet_bytes_with(vec![], Some(payload));
    let (payload_off, sig_len, _, _, payload_body_off, payload_len) = payload_offsets(&bytes);
    let unknown_sig = vec![0xAC; sig_len];
    bytes[payload_off + 1..payload_off + 1 + sig_len].copy_from_slice(&unknown_sig);
    let truncated = bytes[..payload_body_off + payload_len as usize - 1].to_vec();

    let status = <Packet as TryReadPacketFromBuffered>::try_read(
        &mut std::io::Cursor::new(&truncated),
        &mut (),
    )
    .expect("partial packet must not hard fail");
    assert!(matches!(status, PacketReadStatus::NotEnoughData(_)));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_unknown_block_len_is_body_len() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let total_len = block_total_len(&bytes, off);
    let body_len = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap()) as u64;
    let sig = unknown_block_signature(&bytes[off..off + total_len]);
    bytes[off..off + 4].copy_from_slice(&sig);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (_, skipped) = unwrap_packet_success(status);
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].len, Some(body_len));
    assert_eq!(body_len as usize, total_len - 4 - 4 - 4);
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_all_blocks_unknown_returns_empty_packet_blocks() {
    let b1 = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let b2 = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let mut bytes = packet_bytes_with(vec![b1, b2], None);
    let off0 = blocks_offset();
    let off1 = off0 + block_total_len(&bytes, off0);
    let len0 = block_total_len(&bytes, off0);
    let len1 = block_total_len(&bytes, off1);
    let sig0 = unknown_block_signature(&bytes[off0..off0 + len0]);
    let sig1 = unknown_block_signature(&bytes[off1..off1 + len1]);
    bytes[off0..off0 + 4].copy_from_slice(&sig0);
    bytes[off1..off1 + 4].copy_from_slice(&sig1);

    let status = <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
        .expect("read should succeed");
    let (packet, skipped) = unwrap_packet_success(status);
    assert!(packet.blocks.is_empty());
    assert_eq!(packet.payload, None);
    assert_eq!(skipped.len(), 2);
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_storage_reader_nth_skips_unknown_block() {
    let tmp = std::env::temp_dir().join("stress_resilient_storage_reader_nth.bin");
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)
        .unwrap();
    let wrapped = WrappedPacket {
        blocks: vec![
            Block::BlockEnums(BlockEnums {
                level: Level::Err,
                kind: Kind::File,
            }),
            Block::BlockEnums(BlockEnums {
                level: Level::Warn,
                kind: Kind::Stream,
            }),
        ],
        payload: None,
    };
    let mut writer = Writer::new(&mut file).unwrap();
    writer.insert((&wrapped).into(), &mut ()).unwrap();
    drop(writer);
    file.sync_all().unwrap();

    let mut bytes = std::fs::read(&tmp).unwrap();
    let packet_off = PacketHeader::get_pos(&bytes).expect("packet must exist");
    let block_off = packet_off + blocks_offset();
    let block_len = block_total_len(&bytes[packet_off..], blocks_offset());
    let sig = unknown_block_signature(&bytes[block_off..block_off + block_len]);
    bytes[block_off..block_off + 4].copy_from_slice(&sig);
    std::fs::write(&tmp, &bytes).unwrap();

    let file = std::fs::OpenOptions::new().read(true).write(true).open(&tmp).unwrap();
    let mut reader = Reader::new(&file).unwrap();
    let packet = reader.nth(0, &mut ()).unwrap().expect("packet must exist");
    assert_eq!(
        packet.blocks,
        vec![Block::BlockEnums(BlockEnums {
            level: Level::Warn,
            kind: Kind::Stream,
        })]
    );
    assert_eq!(packet.payload, None);
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_corrupted_len_returns_invalid_length() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Err,
        kind: Kind::File,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let mut len = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap());
    len += 1;
    bytes[off + 4..off + 8].copy_from_slice(&len.to_le_bytes());

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail on corrupted known block len"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::InvalidLength));
}

#[cfg(feature = "resilient")]
#[test]
fn resilient_len_overflow_returns_invalid_length() {
    let block = Block::BlockEnums(BlockEnums {
        level: Level::Warn,
        kind: Kind::Stream,
    });
    let mut bytes = packet_bytes_with(vec![block], None);
    let off = blocks_offset();
    let len = block_total_len(&bytes, off);
    let sig = unknown_block_signature(&bytes[off..off + len]);
    bytes[off..off + 4].copy_from_slice(&sig);
    bytes[off + 4..off + 8].copy_from_slice(&u32::MAX.to_le_bytes());

    let err = match <Packet as TryReadPacketFrom>::try_read(&mut std::io::Cursor::new(&bytes), &mut ())
    {
        Ok(_) => panic!("must fail when unknown block len exits packet bounds"),
        Err(err) => err,
    };
    assert!(matches!(err, brec::Error::InvalidLength));
}

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

    #[test]
    fn storage_write_stream_read_no_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(true), 1..max())) {
        storage_write_stream_read(packets, STORAGE_WRITE_STREAM_READ_NO_BLOCKS_FILE)?;
    }

    #[test]
    fn storage_write_stream_read_with_blocks(packets in proptest::collection::vec(WrappedPacket::arbitrary_with(false), 1..max())) {
        storage_write_stream_read(packets, STORAGE_WRITE_STREAM_READ_BLOCKS_FILE)?;
    }

}
