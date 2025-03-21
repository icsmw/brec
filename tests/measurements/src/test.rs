use std::io::Write;

use crate::*;
use brec::prelude::*;
use proptest::prelude::*;
// use std::{
//     fs::File,
//     io::{BufRead, BufReader, Write},
//     time::{Duration, Instant},
// };

brec::include_generated!();

#[derive(Debug)]
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
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<Record>()
            .prop_map(|record| WrappedPacket {
                blocks: vec![Block::Metadata(record.mt)],
                payload: Some(Payload::String(record.msg)),
            })
            .boxed()
    }
}

fn create_text_log_file(rows: Vec<TextualRow>, filename: &str) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = std::env::temp_dir().join(filename);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    for row in rows.iter() {
        file.write_all(format!("{}\n", row.msg,).as_bytes())?;
    }
    file.flush()
}

fn read_text_logs(filename: &str) -> std::io::Result<()> {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        time::Instant,
    };
    let now = Instant::now();
    let tmp = std::env::temp_dir().join(filename);
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    let mut errors = 0;
    let err = Level::Err.to_string();
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(&err) {
            errors += 1;
        }
        count += 1;
    }
    println!(
        "text logs read in {}ms ({count} lines; {errors} logs marked as errors)",
        now.elapsed().as_millis()
    );
    Ok(())
}

fn create_bin_log_file(packets: Vec<WrappedPacket>, filename: &str) -> std::io::Result<()> {
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
    Ok(())
}

fn read_bin_logs(filename: &str) -> std::io::Result<()> {
    use std::{fs::File, time::Instant};
    let now = Instant::now();
    let tmp = std::env::temp_dir().join(filename);
    let mut file: File = File::open(tmp)?;
    let mut storage = Storage::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    storage
        .add_rule(Rule::FilterByBlocks(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks: &[BlockReferred]| {
                blocks.iter().any(|bl| {
                    if let BlockReferred::Metadata(bl) = bl {
                        matches!(bl.level, Level::Err)
                    } else {
                        false
                    }
                })
            },
        ))))
        .unwrap();
    let mut count = 0;
    for packet in storage.iter() {
        match packet {
            Ok(_packet) => {
                count += 1;
            }
            Err(err) => {
                panic!("Fail to read storage: {err}");
            }
        }
    }
    println!(
        "bin logs read in {}ms ({count} lines)",
        now.elapsed().as_millis()
    );
    Ok(())
}

fn create_bin_log_stream_file(packets: Vec<WrappedPacket>, filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(&mut file)?;
    }
    file.flush()
}

fn read_bin_stream_logs(filename: &str) -> std::io::Result<()> {
    use std::{fs::File, time::Instant};
    let tmp = std::env::temp_dir().join(filename);
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    let mut count = 0;
    loop {
        match reader.read() {
            Ok(next) => match next {
                NextPacket::Found(_packet) => count += 1,
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
    println!(
        "bin stream logs read in {}ms ({count} lines)",
        now.elapsed().as_millis()
    );
    Ok(())
}

const TEXT_LOG_FILE: &str = "test_measurements.log";
const BIN_LOG_FILE: &str = "test_measurements.bin";
const BIN_STREAM_LOG_FILE: &str = "test_measurements_stream.bin";

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(1)
    })]


    #[test]
    fn text_logs(rows in proptest::collection::vec(any::<TextualRow>(), 100_000)) {
        create_text_log_file(rows, TEXT_LOG_FILE)?;
        read_text_logs(TEXT_LOG_FILE)?;
    }

    #[test]
    fn bin_logs(rows in proptest::collection::vec(any::<WrappedPacket>(), 100_000)) {
        create_bin_log_file(rows, BIN_LOG_FILE)?;
        read_bin_logs(BIN_LOG_FILE)?;
    }

    #[test]
    fn bin_logs_stream(rows in proptest::collection::vec(any::<WrappedPacket>(), 100_000)) {
        create_bin_log_stream_file(rows, BIN_STREAM_LOG_FILE)?;
        read_bin_stream_logs(BIN_STREAM_LOG_FILE)?;
    }


}
