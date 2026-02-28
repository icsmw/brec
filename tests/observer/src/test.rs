use brec::prelude::*;
use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::strategy::ValueTree;

use crate::*;

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

#[test]
fn storage_write_read_filter() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(10 + 1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!("brec_test_{}.tmp", std::process::id());
    let tmp = std::env::temp_dir().join(filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let mut writer = Writer::new(&mut wfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    const CHECKPOINTS: [usize; 13] = [
        0,
        1,
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_div(2),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_sub(10),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_sub(1),
        brec::storage::DEFAULT_SLOT_CAPACITY,
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_add(1),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_add(10),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_sub(10),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_sub(1),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_mul(2),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_add(1),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_add(10),
    ];

    let rfile = std::fs::OpenOptions::new().read(true).open(&tmp)?;

    let mut reader = Reader::new(&rfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut last = 0;
    let mut total = 0;
    for (idx, packet) in packets.iter().enumerate() {
        if CHECKPOINTS.contains(&idx) {
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            println!("Checkpoint {idx}: added {added} packets");
            assert_eq!(added + last, idx);
            // Repeated reload should not add more packets
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            assert_eq!(added, 0);
            assert_eq!(reader.count(), idx);
            // Try to seek to the last packet, which should be valid
            match reader.seek(last) {
                Err(Error::EmptySource) => {}
                Err(err) => panic!("Unexpected error on seek to {last} from {idx}: {err}"),
                Ok(mut iterator) => {
                    let mut read = 0;
                    // Check packets from the last checkpoint to the current index
                    for (i, expected) in packets[last..idx].iter().enumerate() {
                        read += 1;
                        let Some(pkg) = iterator.next() else {
                            panic!("Expected packet at index {} but got None", last + i);
                        };
                        if let Err(err) = pkg {
                            panic!("Error reading packet at index {}: {err}", last + i);
                        } else {
                            let pkg = pkg.unwrap();
                            let wrapped: WrappedPacket = pkg.into();
                            assert_eq!(
                                &wrapped,
                                expected,
                                "Packet mismatch at index {}: expected {:?}, got {:?}",
                                last + i,
                                expected,
                                wrapped
                            );
                        }
                    }
                    assert!(
                        iterator.next().is_none(),
                        "Expected no more packets after index {}, but got some",
                        idx - 1
                    );
                    println!(
                        "\t- has been read {} packets from index {} to {}; all packets match as expected.",
                        read,
                        last,
                        idx - 1
                    );
                    total += read;
                }
            };
            last = idx;
        }
        writer
            .insert(packet.into())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    // Last package is not expected to be read since it is added after the last checkpoint
    assert_eq!(total, packets.len() - 1);
    Ok(())
}

fn gen_n<T: Arbitrary>(n: usize) -> Vec<T> {
    let mut runner = proptest::test_runner::TestRunner::default();
    let strat = any::<T>();

    (0..n)
        .map(|_| strat.new_tree(&mut runner).unwrap().current())
        .collect()
}
