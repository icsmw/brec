use crate::test::*;
use crate::*;
use std::{
    fs::{File, metadata},
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

pub fn read_file(payload: report::PayloadKind, filename: &str) -> std::io::Result<()> {
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
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
        return Err(std::io::Error::other(format!(
            "Dismatch lengths: {} vs {count}",
            storage.count()
        )));
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Storage,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn filter_file(payload: report::PayloadKind, filename: &str) -> std::io::Result<()> {
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    storage
        .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks| {
                blocks
                    .find::<Metadata, _>(|bl| matches!(bl.level, Level::Err))
                    .is_some()
            },
        ))))
        .unwrap();
    storage
        .add_rule(Rule::FilterPacket(brec::RuleFnDef::Dynamic(Box::new(
            tests::is_match_packet,
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
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Storage,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}
