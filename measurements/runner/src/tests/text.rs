use crate::test::MATCH;
use crate::*;
use std::io::{Read, Write};
use std::{
    fs::{File, metadata},
    io::{BufRead, BufReader},
    time::Instant,
};

pub fn create_file(
    payload: report::PayloadKind,
    rows: Vec<TextualRow>,
    mut count: usize,
    filename: &str,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let size_rows = rows.len();
    let total_count = size_rows * count;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    while count > 0 {
        for row in rows.iter() {
            file.write_all(format!("{}\n", row.msg,).as_bytes())?;
        }
        count -= 1;
    }
    file.flush()?;
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Text,
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
    rows: Vec<TextualRow>,
    mut count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let size_rows = rows.len();
    let total_count = size_rows * count;
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    let mut encrypt =
        crate::tests::crypt::encrypt_options(session_reuse_limit, decrypt_cache_limit);
    while count > 0 {
        for row in rows.iter() {
            let encrypted = crate::tests::crypt::encrypt_bytes(row.msg.as_bytes(), &mut encrypt)?;
            file.write_all(&(encrypted.len() as u32).to_le_bytes())?;
            file.write_all(&encrypted)?;
        }
        count -= 1;
    }
    file.flush()?;
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::TextCrypt,
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
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line_result in reader.lines() {
        let _line = line_result?;
        count += 1;
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Text,
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
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    let err = Level::Err.to_string();
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(&err) && line.contains(MATCH) {
            count += 1;
        }
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Text,
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
    let mut file = File::open(tmp)?;
    let mut decrypt =
        crate::tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;
    loop {
        let mut len = [0u8; 4];
        match file.read_exact(&mut len) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(err),
        }
        let len = u32::from_le_bytes(len) as usize;
        let mut encrypted = vec![0u8; len];
        file.read_exact(&mut encrypted)?;
        let decrypted = crate::tests::crypt::decrypt_bytes(&encrypted, &mut decrypt)?;
        let _line = String::from_utf8(decrypted)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        count += 1;
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::TextCrypt,
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
    let mut file = File::open(tmp)?;
    let mut decrypt =
        crate::tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;
    let err = Level::Err.to_string();
    loop {
        let mut len = [0u8; 4];
        match file.read_exact(&mut len) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(err),
        }
        let len = u32::from_le_bytes(len) as usize;
        let mut encrypted = vec![0u8; len];
        file.read_exact(&mut encrypted)?;
        let decrypted = crate::tests::crypt::decrypt_bytes(&encrypted, &mut decrypt)?;
        let line = String::from_utf8(decrypted)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        if line.contains(&err) && line.contains(MATCH) {
            count += 1;
        }
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::TextCrypt,
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
