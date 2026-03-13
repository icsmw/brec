use super::codec::{decode_borrowed_for_read, decode_for_read, packet_to_pb, pb_to_bytes, record_matches};
use crate::test::WrappedPacket;
use crate::*;
use std::fs::{File, metadata};
use std::io::{Read, Write};
use std::time::Instant;

fn read_len_prefixed(file: &mut File) -> std::io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match file.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err),
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut data = vec![0u8; len];
    file.read_exact(&mut data)?;
    Ok(Some(data))
}

pub fn create_file(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
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

    while count > 0 {
        for packet in packets.iter() {
            let row = match payload {
                report::PayloadKind::RecordCrypt => packet_to_pb(payload, packet, Some(&mut encrypt))?,
                _ => packet_to_pb(payload, packet, None)?,
            };
            let encoded = pb_to_bytes(&row)?;
            file.write_all(&(encoded.len() as u32).to_le_bytes())?;
            file.write_all(&encoded)?;
        }
        count -= 1;
    }

    file.flush()?;
    let size = metadata(&tmp).expect("Read File Meta").len();
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Protobuf,
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

pub fn read_file(
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
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;

    while let Some(data) = read_len_prefixed(&mut file)? {
        decode_for_read(payload, data.as_slice(), &mut decrypt)?;
        count += 1;
    }

    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Protobuf,
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
    let mut file = File::open(tmp)?;
    let mut count = 0usize;

    while let Some(data) = read_len_prefixed(&mut file)? {
        decode_borrowed_for_read(data.as_slice())?;
        count += 1;
    }

    assert_eq!(count, expected_count);
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Protobuf,
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

pub fn filter_file(
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
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;

    while let Some(data) = read_len_prefixed(&mut file)? {
        let matched = match payload {
            report::PayloadKind::RecordCrypt => {
                record_matches(payload, data.as_slice(), Some(&mut decrypt))?
            }
            _ => record_matches(payload, data.as_slice(), None)?,
        };
        if matched {
            count += 1;
        }
    }

    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Protobuf,
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
