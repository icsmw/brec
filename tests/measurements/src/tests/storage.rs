use crate::test::*;
use crate::*;
use std::{
    fs::{metadata, File},
    time::Instant,
};

pub fn create_file(
    packets: Vec<WrappedPacket>,
    mut count: usize,
    filename: &str,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut storage = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    while count > 0 {
        for packet in packets.iter() {
            storage.insert(packet.into()).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
        }
        count -= 1;
    }
    Ok(())
}

pub fn read_file(filename: &str) -> std::io::Result<()> {
    let now = Instant::now();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
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
    if count != storage.count() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Dismatch lengths: {} vs {count}", storage.count()),
        ));
    }
    report::add(
        report::Platform::Storage,
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
    let now = Instant::now();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    storage
        .add_rule(Rule::FilterByBlocks(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks: &[BlockReferred]| {
                blocks.iter().any(|bl| {
                    let BlockReferred::Metadata(bl) = bl;
                    matches!(bl.level, Level::Err)
                })
            },
        ))))
        .unwrap();
    storage
        .add_rule(Rule::FilterByPayload(brec::RuleFnDef::Dynamic(Box::new(
            move |pl: &[u8]| {
                std::str::from_utf8(pl)
                    .expect("Valid string")
                    .contains(MATCH)
            },
        ))))
        .unwrap();
    let mut count = 0;
    for packet in storage.filtered() {
        match packet {
            Ok(_packet) => {
                count += 1;
            }
            Err(err) => {
                panic!("Fail to read storage: {err}");
            }
        }
    }
    report::add(
        report::Platform::Storage,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
        },
    );
    Ok(())
}
