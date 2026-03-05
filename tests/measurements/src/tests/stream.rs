use crate::test::*;
use crate::*;
use brec::prelude::*;
use std::io::Write;
use std::{
    fs::{File, metadata},
    time::Instant,
};

pub fn create_file(
    packets: &[WrappedPacket],
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
    while count > 0 {
        for wrapped in packets.iter() {
            let mut packet: Packet = wrapped.into();
            packet.write_all(&mut file)?;
        }
        count -= 1;
    }
    file.flush()
}

pub fn read_file(payload: report::PayloadKind, filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
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
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BinStream,
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
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    reader
        .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks| {
                blocks
                    .find::<Metadata, _>(|bl| matches!(bl.level, Level::Err))
                    .is_some()
            },
        ))))
        .unwrap();
    reader
        .add_rule(Rule::FilterPacket(brec::RuleFnDef::Dynamic(Box::new(
            tests::is_match_packet,
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
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BinStream,
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
