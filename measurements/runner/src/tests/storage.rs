use crate::test::*;
use crate::*;
use std::{
    fs::{File, metadata},
    time::Instant,
};

pub fn create_file(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
    count: usize,
    filename: &str,
) -> std::io::Result<()> {
    create_file_for_platform(payload, packets, count, filename, report::Platform::BrecStorage)
}

pub(crate) fn create_file_for_platform(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
    mut count: usize,
    filename: &str,
    platform: report::Platform,
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
    let mut storage = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut ctx = PayloadContext::None;
    while count > 0 {
        for packet in packets.iter() {
            storage.insert(packet.into(), &mut ctx).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
        }
        count -= 1;
    }
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        platform,
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
    packets: Vec<WrappedPacket>,
    count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    create_file_crypt_for_platform(
        payload,
        packets,
        count,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::BrecStorage,
    )
}

pub(crate) fn create_file_crypt_for_platform(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
    mut count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
    platform: report::Platform,
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
    let mut storage = Writer::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut encrypt_ctx = PayloadContext::Encrypt(&mut encrypt);
    while count > 0 {
        for packet in packets.iter() {
            storage
                .insert(packet.into(), &mut encrypt_ctx)
                .map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?;
        }
        count -= 1;
    }
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        platform,
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
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut ctx = PayloadContext::None;
    let mut count = 0;
    for packet in storage.iter(&mut ctx) {
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
        report::Platform::BrecStorage,
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

pub fn read_file_borrowed(
    payload: report::PayloadKind,
    filename: &str,
    expected_count: usize,
) -> std::io::Result<()> {
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut ctx = PayloadContext::None;
    let mut count = 0usize;
    for packet in storage.iter(&mut ctx) {
        match packet {
            Ok(_packet) => count += 1,
            Err(err) => panic!("Fail to read storage: {err}"),
        }
    }
    assert_eq!(count, expected_count);
    if count != storage.count() {
        return Err(std::io::Error::other(format!(
            "Dismatch lengths: {} vs {count}",
            storage.count()
        )));
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::BrecStorage,
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
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut storage = Reader::new(&mut file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);
    let mut count = 0;
    for packet in storage.iter(&mut decrypt_ctx) {
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
        report::Platform::BrecStorage,
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
    let mut ctx = PayloadContext::None;
    let mut count = 0;
    for packet in storage.filtered(&mut ctx) {
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
        report::Platform::BrecStorage,
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
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file: File = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
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
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);
    let mut count = 0;
    for packet in storage.filtered(&mut decrypt_ctx) {
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
        report::Platform::BrecStorage,
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
