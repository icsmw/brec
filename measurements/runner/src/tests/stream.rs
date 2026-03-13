use crate::test::*;
use crate::*;
use brec::prelude::*;
use std::io::Write;
use std::{
    fs::{File, metadata},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

pub fn create_file(
    payload: report::PayloadKind,
    packets: &[WrappedPacket],
    mut count: usize,
    filename: &str,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let packet_count = packets.len();
    let total_count = packet_count * count;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut ctx = PayloadContext::None;
    while count > 0 {
        for wrapped in packets.iter() {
            let mut packet: Packet = wrapped.into();
            packet.write_all(&mut file, &mut ctx)?;
        }
        count -= 1;
    }
    file.flush()?;
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BrecStream,
        report::TestCase::Writing,
        report::TestResults {
            size,
            count: total_count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn create_file_crypt(
    payload: report::PayloadKind,
    packets: &[WrappedPacket],
    mut count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let packet_count = packets.len();
    let total_count = packet_count * count;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut encrypt = tests::crypt::encrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut encrypt_ctx = PayloadContext::Encrypt(&mut encrypt);
    while count > 0 {
        for wrapped in packets.iter() {
            let mut packet: Packet = wrapped.into();
            packet.write_all(&mut file, &mut encrypt_ctx)?;
        }
        count -= 1;
    }
    file.flush()?;
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BrecStream,
        report::TestCase::Writing,
        report::TestResults {
            size,
            count: total_count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn read_file(payload: report::PayloadKind, filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    let mut ctx = PayloadContext::None;
    let mut count = 0;
    loop {
        match reader.read(&mut ctx) {
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
        report::Platform::BrecStream,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn read_file_crypt(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file: File = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);
    let mut count = 0;
    loop {
        match reader.read(&mut decrypt_ctx) {
            Ok(next) => match next {
                NextPacket::Found(_packet) => count += 1,
                NextPacket::NotFound => {}
                NextPacket::NotEnoughData(_needed) => {}
                NextPacket::NoData => break,
                NextPacket::Skipped => {}
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
        report::Platform::BrecStream,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_nanos(),
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
    let mut ctx = PayloadContext::None;
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
        match reader.read(&mut ctx) {
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
        report::Platform::BrecStream,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn filter_file_crypt(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file: File = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);
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
        match reader.read(&mut decrypt_ctx) {
            Ok(next) => match next {
                NextPacket::Found(_packet) => count += 1,
                NextPacket::NotFound => {}
                NextPacket::NotEnoughData(_needed) => {}
                NextPacket::NoData => break,
                NextPacket::Skipped => {}
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
        report::Platform::BrecStream,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}

pub fn read_file_borrowed_referred(
    payload: report::PayloadKind,
    filename: &str,
    expected_count: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(&mut file);
    let mut ctx = PayloadContext::None;
    let visited = Arc::new(AtomicUsize::new(0));
    let visited_ref = Arc::clone(&visited);
    reader
        .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
            move |blocks| {
                let _ = blocks.get::<BlockBorrowed>().map(|block| {
                    let _ = (
                        block.field_u8,
                        block.field_u16,
                        block.field_u32,
                        block.field_u64,
                        block.field_u128,
                        block.field_i8,
                        block.field_i16,
                        block.field_i32,
                        block.field_i64,
                        block.field_i128,
                        block.field_f32,
                        block.field_f64,
                        block.field_bool,
                        block.blob_a[0],
                        block.blob_b[0],
                    );
                    visited_ref.fetch_add(1, Ordering::Relaxed);
                });
                false
            },
        ))))
        .unwrap();
    loop {
        match reader.read(&mut ctx) {
            Ok(next) => match next {
                NextPacket::Found(_packet) => {}
                NextPacket::NotFound => {}
                NextPacket::NotEnoughData(_needed) => {}
                NextPacket::NoData => break,
                NextPacket::Skipped => {}
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
    let visited_count = visited.load(Ordering::Relaxed);
    assert_eq!(visited_count, expected_count);
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BrecStream,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count: visited_count,
            time: now.elapsed().as_nanos(),
            cpu_ms: usage.cpu_ms,
            rss_kb: usage.rss_kb,
            peak_rss_kb: usage.peak_rss_kb,
        },
    );
    Ok(())
}
