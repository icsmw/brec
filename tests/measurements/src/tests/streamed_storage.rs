use crate::test::*;
use crate::*;
use brec::prelude::*;
use std::{
    fs::{metadata, File},
    time::Instant,
};

pub fn create_file(
    packets: Vec<WrappedPacket>,
    count: usize,
    filename: &str,
) -> std::io::Result<()> {
    tests::storage::create_file(packets, count, filename)
}

pub fn read_file(filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
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
    report::add(
        report::Platform::StreamedStorage,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
        },
    );
    Ok(())
}

pub fn filter_file(filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    reader
        .add_rule(Rule::FilterByBlocks(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks: &[BlockReferred]| {
                blocks.iter().any(|bl| {
                    let BlockReferred::Metadata(bl) = bl;
                    matches!(bl.level, Level::Err)
                })
            },
        ))))
        .unwrap();
    reader
        .add_rule(Rule::FilterByPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |pl: &[u8]| {
                std::str::from_utf8(pl)
                    .expect("Valid string")
                    .contains(MATCH)
            },
        ))))
        .unwrap();
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
    report::add(
        report::Platform::StreamedStorage,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
        },
    );
    Ok(())
}
