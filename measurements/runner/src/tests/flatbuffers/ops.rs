use super::common::{
    VT_RECORD_BINARY_META, VT_RECORD_BINARY_PAYLOAD, VT_RECORD_BORROWED_BLOCK, VT_RECORD_CRYPT_META,
    VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED, VT_RECORD_META, VT_RECORD_MSG, fb_table, fb_table_safe,
    get_required_bytes, get_required_str, get_required_table,
};
use super::decode::{
    attachment_name, decode_attachment_owned, decode_borrowed_owned, decode_meta_owned,
    read_meta_level, touch_attachment, touch_borrowed_block, touch_meta,
};
use super::encode::record_bytes_from_packet;
use crate::test::{MATCH, WrappedPacket};
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

fn create_file_for_platform(
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
    while count > 0 {
        for packet in packets.iter() {
            let encoded = record_bytes_from_packet(payload, packet, &mut encrypt)?;
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

pub fn create_file(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
    count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    create_file_for_platform(
        payload,
        packets,
        count,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffers,
    )
}

pub fn create_file_safe(
    payload: report::PayloadKind,
    packets: Vec<WrappedPacket>,
    count: usize,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    create_file_for_platform(
        payload,
        packets,
        count,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffersOwned,
    )
}

fn read_file_for_platform(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
    platform: report::Platform,
    safe_mode: bool,
) -> std::io::Result<()> {
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;
    while let Some(data) = read_len_prefixed(&mut file)? {
        let root = if safe_mode {
            fb_table_safe(data.as_slice())?
        } else {
            fb_table(data.as_slice())
        };
        if safe_mode {
            match payload {
                report::PayloadKind::Record => {
                    let meta = get_required_table(&root, VT_RECORD_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let _msg = get_required_str(&root, VT_RECORD_MSG, "missing record.msg")?.to_owned();
                    let _ = (owned_meta.level, owned_meta.target, owned_meta.tm);
                }
                report::PayloadKind::RecordBincode => {
                    let meta = get_required_table(&root, VT_RECORD_BINARY_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let attachment =
                        get_required_table(&root, VT_RECORD_BINARY_PAYLOAD, "missing record_binary.payload")?;
                    let owned_attachment = decode_attachment_owned(&attachment)?;
                    let _ = (
                        owned_meta.level,
                        owned_meta.target,
                        owned_meta.tm,
                        owned_attachment.uuid.as_str(),
                        owned_attachment.name.as_str(),
                        owned_attachment.chunk,
                        owned_attachment.data.len(),
                        owned_attachment.fields.len(),
                    );
                }
                report::PayloadKind::RecordCrypt => {
                    let meta = get_required_table(&root, VT_RECORD_CRYPT_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let encrypted = get_required_bytes(
                        &root,
                        VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED,
                        "missing record_crypt.payload_encrypted",
                    )?;
                    let decrypted_payload = tests::crypt::decrypt_bytes(encrypted, &mut decrypt)?;
                    let attachment = fb_table_safe(decrypted_payload.as_slice())?;
                    let owned_attachment = decode_attachment_owned(&attachment)?;
                    let _ = (
                        owned_meta.level,
                        owned_meta.target,
                        owned_meta.tm,
                        owned_attachment.uuid.as_str(),
                        owned_attachment.name.as_str(),
                        owned_attachment.chunk,
                        owned_attachment.data.len(),
                        owned_attachment.fields.len(),
                    );
                }
                report::PayloadKind::Borrowed => {
                    let block = get_required_table(&root, VT_RECORD_BORROWED_BLOCK, "missing borrowed block")?;
                    let owned = decode_borrowed_owned(&block)?;
                    let _ = (
                        owned.field_u8,
                        owned.field_u16,
                        owned.field_u32,
                        owned.field_u64,
                        owned.field_u128,
                        owned.field_i8,
                        owned.field_i16,
                        owned.field_i32,
                        owned.field_i64,
                        owned.field_i128,
                        owned.field_f32,
                        owned.field_f64,
                        owned.field_bool,
                        owned.blob_a.len(),
                        owned.blob_b.len(),
                    );
                }
            }
        } else {
            match payload {
                report::PayloadKind::Record => {
                    touch_meta(&root, VT_RECORD_META)?;
                    let _ = get_required_str(&root, VT_RECORD_MSG, "missing record.msg")?;
                }
                report::PayloadKind::RecordBincode => {
                    touch_meta(&root, VT_RECORD_BINARY_META)?;
                    let attachment =
                        get_required_table(&root, VT_RECORD_BINARY_PAYLOAD, "missing record_binary.payload")?;
                    touch_attachment(&attachment)?;
                }
                report::PayloadKind::RecordCrypt => {
                    touch_meta(&root, VT_RECORD_CRYPT_META)?;
                    let encrypted = get_required_bytes(
                        &root,
                        VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED,
                        "missing record_crypt.payload_encrypted",
                    )?;
                    let decrypted_payload = tests::crypt::decrypt_bytes(encrypted, &mut decrypt)?;
                    let attachment = fb_table(decrypted_payload.as_slice());
                    touch_attachment(&attachment)?;
                }
                report::PayloadKind::Borrowed => {
                    let block = get_required_table(&root, VT_RECORD_BORROWED_BLOCK, "missing borrowed block")?;
                    touch_borrowed_block(&block)?;
                }
            }
        }
        count += 1;
    }
    let usage = metrics.finish();
    report::add(
        payload,
        platform,
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

pub fn read_file(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    read_file_for_platform(
        payload,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffers,
        false,
    )
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
        let root = fb_table(data.as_slice());
        let block = get_required_table(&root, VT_RECORD_BORROWED_BLOCK, "missing borrowed block")?;
        touch_borrowed_block(&block)?;
        count += 1;
    }
    assert_eq!(count, expected_count);
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::FlatBuffers,
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

pub fn read_file_safe(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    read_file_for_platform(
        payload,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffersOwned,
        true,
    )
}

fn filter_file_for_platform(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
    platform: report::Platform,
    safe_mode: bool,
) -> std::io::Result<()> {
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let mut file = File::open(tmp)?;
    let mut decrypt = tests::crypt::decrypt_options(session_reuse_limit, decrypt_cache_limit);
    let mut count = 0;
    let err_level = u8::from(&Level::Err);

    while let Some(data) = read_len_prefixed(&mut file)? {
        let root = if safe_mode {
            fb_table_safe(data.as_slice())?
        } else {
            fb_table(data.as_slice())
        };
        let matched = if safe_mode {
            match payload {
                report::PayloadKind::Record => {
                    let meta = get_required_table(&root, VT_RECORD_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let msg = get_required_str(&root, VT_RECORD_MSG, "missing record.msg")?.to_owned();
                    owned_meta.level == err_level && msg.contains(MATCH)
                }
                report::PayloadKind::RecordBincode => {
                    let meta = get_required_table(&root, VT_RECORD_BINARY_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let attachment =
                        get_required_table(&root, VT_RECORD_BINARY_PAYLOAD, "missing record_binary.payload")?;
                    let owned_attachment = decode_attachment_owned(&attachment)?;
                    owned_meta.level == err_level && owned_attachment.name.contains(MATCH)
                }
                report::PayloadKind::RecordCrypt => {
                    let meta = get_required_table(&root, VT_RECORD_CRYPT_META, "missing meta table")?;
                    let owned_meta = decode_meta_owned(&meta)?;
                    let encrypted = get_required_bytes(
                        &root,
                        VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED,
                        "missing record_crypt.payload_encrypted",
                    )?;
                    let decrypted_payload = tests::crypt::decrypt_bytes(encrypted, &mut decrypt)?;
                    let attachment = fb_table_safe(decrypted_payload.as_slice())?;
                    let owned_attachment = decode_attachment_owned(&attachment)?;
                    owned_meta.level == err_level && owned_attachment.name.contains(MATCH)
                }
                report::PayloadKind::Borrowed => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Borrowed payload is not supported in filtering scenario",
                    ));
                }
            }
        } else {
            match payload {
                report::PayloadKind::Record => {
                    if read_meta_level(&root, VT_RECORD_META)? != err_level {
                        false
                    } else {
                        get_required_str(&root, VT_RECORD_MSG, "missing record.msg")?.contains(MATCH)
                    }
                }
                report::PayloadKind::RecordBincode => {
                    if read_meta_level(&root, VT_RECORD_BINARY_META)? != err_level {
                        false
                    } else {
                        let attachment =
                            get_required_table(&root, VT_RECORD_BINARY_PAYLOAD, "missing record_binary.payload")?;
                        attachment_name(&attachment)?.contains(MATCH)
                    }
                }
                report::PayloadKind::RecordCrypt => {
                    if read_meta_level(&root, VT_RECORD_CRYPT_META)? != err_level {
                        false
                    } else {
                        let encrypted = get_required_bytes(
                            &root,
                            VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED,
                            "missing record_crypt.payload_encrypted",
                        )?;
                        let decrypted_payload = tests::crypt::decrypt_bytes(encrypted, &mut decrypt)?;
                        let attachment = fb_table(decrypted_payload.as_slice());
                        attachment_name(&attachment)?.contains(MATCH)
                    }
                }
                report::PayloadKind::Borrowed => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Borrowed payload is not supported in filtering scenario",
                    ));
                }
            }
        };
        if matched {
            count += 1;
        }
    }

    let usage = metrics.finish();
    report::add(
        payload,
        platform,
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

pub fn filter_file(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    filter_file_for_platform(
        payload,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffers,
        false,
    )
}

pub fn filter_file_safe(
    payload: report::PayloadKind,
    filename: &str,
    session_reuse_limit: u32,
    decrypt_cache_limit: usize,
) -> std::io::Result<()> {
    filter_file_for_platform(
        payload,
        filename,
        session_reuse_limit,
        decrypt_cache_limit,
        report::Platform::FlatBuffersOwned,
        true,
    )
}
