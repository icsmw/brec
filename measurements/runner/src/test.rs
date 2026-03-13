use crate::*;
use brec::crypt::CryptPolicy;
use proptest::{prelude::*, strategy::ValueTree};
use std::path::PathBuf;

pub const MATCH: &str = "-match-";

brec::generate!();

struct TestConfiguration {
    pub iterations_crypt: usize,
    pub iterations: usize,
    pub packages: usize,
    pub records: usize,
    pub session_reuse_limit: u32,
    pub decrypt_cache_limit: usize,
    pub measurements_cvs: Option<String>,
}

impl Default for TestConfiguration {
    fn default() -> Self {
        Self {
            iterations_crypt: 10,
            iterations: 10,
            packages: 100,
            records: 10000,
            session_reuse_limit: CryptPolicy::default().session_reuse_limit,
            decrypt_cache_limit: CryptPolicy::default().decrypt_cache_limit,
            measurements_cvs: None,
        }
    }
}

impl TestConfiguration {
    pub fn new() -> Self {
        const ITERATIONS: &str = "BREC_TEST_MEASUREMENTS_ITERATIONS";
        const ITERATIONS_CRYPT: &str = "BREC_TEST_MEASUREMENTS_ITERATIONS_CRYPT";
        const PACKAGES: &str = "BREC_TEST_MEASUREMENTS_PACKAGES";
        const RECORDS: &str = "BREC_TEST_MEASUREMENTS_RECORDS";
        const SESSION_REUSE_LIMIT: &str = "BREC_SESSION_REUSE_LIMIT";
        const DECRYPT_CACHE_LIMIT: &str = "BREC_DECRYPT_CACHE_LIMIT";
        const MEASUREMENTS_CVS: &str = "BREC_TEST_MEASUREMENTS_CVS";

        let defaults = Self::default();
        Self {
            iterations_crypt: std::env::var(ITERATIONS_CRYPT)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.iterations_crypt))
                .unwrap_or(defaults.iterations_crypt),
            iterations: std::env::var(ITERATIONS)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.iterations))
                .unwrap_or(defaults.iterations),
            packages: std::env::var(PACKAGES)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.packages))
                .unwrap_or(defaults.packages),
            records: std::env::var(RECORDS)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.records))
                .unwrap_or(defaults.records),
            session_reuse_limit: std::env::var(SESSION_REUSE_LIMIT)
                .map(|v| v.parse::<u32>().unwrap_or(defaults.session_reuse_limit))
                .unwrap_or(defaults.session_reuse_limit),
            decrypt_cache_limit: std::env::var(DECRYPT_CACHE_LIMIT)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.decrypt_cache_limit))
                .unwrap_or(defaults.decrypt_cache_limit),
            measurements_cvs: std::env::var(MEASUREMENTS_CVS)
                .ok()
                .and_then(|v| {
                    let trimmed = v.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrappedPacket {
    pub(crate) blocks: Vec<Block>,
    pub(crate) payload: Option<Payload>,
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

fn text_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.log", payload.file_tag())
}

fn json_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.json", payload.file_tag())
}

fn bin_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.bin", payload.file_tag())
}

fn bin_stream_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}_stream.bin", payload.file_tag())
}

fn protobuf_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.protobuf", payload.file_tag())
}

fn flatbuffers_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.flatbuffers", payload.file_tag())
}

fn flatbuffers_safe_log_file(payload: report::PayloadKind) -> String {
    format!("test_measurements_{}.flatbuffers_owned", payload.file_tag())
}

struct TempFileGuard {
    path: PathBuf,
}

impl TempFileGuard {
    fn new(filename: &str) -> Self {
        let path = std::env::temp_dir().join(filename);
        let _ = std::fs::remove_file(&path);
        Self { path }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn run_text_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let rows = rows.iter().map(|row| row.into_text_row()).collect();
    let filename = text_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    if matches!(payload, report::PayloadKind::RecordCrypt) {
        tests::text::create_file_crypt(
            payload,
            rows,
            cfg.records,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::text::read_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::text::filter_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
    } else {
        tests::text::create_file(payload, rows, cfg.records, &filename)?;
        tests::text::read_file(payload, &filename)?;
        tests::text::filter_file(payload, &filename)?;
    }
    Ok(())
}

fn run_json_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let rows = rows.iter().map(|row| row.into_json_row()).collect();
    let filename = json_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    if matches!(payload, report::PayloadKind::RecordCrypt) {
        tests::json::create_file_crypt(
            payload,
            rows,
            cfg.records,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::json::read_file_crypt::<R::JsonPayload>(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::json::filter_file_crypt::<R::JsonPayload>(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
    } else {
        tests::json::create_file(payload, rows, cfg.records, &filename)?;
        tests::json::read_file::<R::JsonPayload>(payload, &filename)?;
        tests::json::filter_file::<R::JsonPayload>(payload, &filename)?;
    }
    Ok(())
}

fn run_bin_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let rows = rows.iter().map(|row| row.into_packet()).collect();
    let filename = bin_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    if matches!(payload, report::PayloadKind::RecordCrypt) {
        tests::storage::create_file_crypt(
            payload,
            rows,
            cfg.records,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::storage::read_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::storage::filter_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
    } else {
        tests::storage::create_file(payload, rows, cfg.records, &filename)?;
        tests::storage::read_file(payload, &filename)?;
        tests::storage::filter_file(payload, &filename)?;
    }
    Ok(())
}

fn run_bin_stream_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let rows: Vec<WrappedPacket> = rows.iter().map(|row| row.into_packet()).collect();
    let filename = bin_stream_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    if matches!(payload, report::PayloadKind::RecordCrypt) {
        tests::stream::create_file_crypt(
            payload,
            &rows,
            cfg.records,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::stream::read_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
        tests::stream::filter_file_crypt(
            payload,
            &filename,
            cfg.session_reuse_limit,
            cfg.decrypt_cache_limit,
        )?;
    } else {
        tests::stream::create_file(payload, &rows, cfg.records, &filename)?;
        tests::stream::read_file(payload, &filename)?;
        tests::stream::filter_file(payload, &filename)?;
    }
    Ok(())
}

fn run_protobuf_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let packets = rows.iter().map(|row| row.into_packet()).collect();
    let filename = protobuf_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    tests::protobuf::create_file(
        payload,
        packets,
        cfg.records,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::protobuf::read_file(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::protobuf::filter_file(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    Ok(())
}

fn run_flatbuffers_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let packets = rows.iter().map(|row| row.into_packet()).collect();
    let filename = flatbuffers_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    tests::flatbuffers::create_file(
        payload,
        packets,
        cfg.records,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::flatbuffers::read_file(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::flatbuffers::filter_file(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    Ok(())
}

fn run_flatbuffers_owned_logs<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let packets = rows.iter().map(|row| row.into_packet()).collect();
    let filename = flatbuffers_safe_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    tests::flatbuffers::create_file_safe(
        payload,
        packets,
        cfg.records,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::flatbuffers::read_file_safe(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::flatbuffers::filter_file_safe(
        payload,
        &filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    Ok(())
}

fn gen_n<T: Arbitrary>(n: usize) -> Vec<T> {
    let mut runner = proptest::test_runner::TestRunner::default();
    let strat = any::<T>();

    (0..n)
        .map(|_| strat.new_tree(&mut runner).unwrap().current())
        .collect()
}

fn runner<T: MeasurementRecord>(rows: Vec<T>, cfg: &TestConfiguration) -> std::io::Result<()> {
    run_text_logs::<T>(&rows, cfg)?;
    run_json_logs::<T>(&rows, cfg)?;
    run_protobuf_logs::<T>(&rows, cfg)?;
    run_flatbuffers_logs::<T>(&rows, cfg)?;
    run_flatbuffers_owned_logs::<T>(&rows, cfg)?;
    run_bin_logs::<T>(&rows, cfg)?;
    run_bin_stream_logs::<T>(&rows, cfg)?;
    Ok(())
}

fn run_borrowed_logs(blocks: &[BlockBorrowed], cfg: &TestConfiguration) -> std::io::Result<()> {
    let payload = report::PayloadKind::Borrowed;
    let expected_count = blocks.len() * cfg.records;
    let packets: Vec<WrappedPacket> = blocks
        .iter()
        .map(|block| WrappedPacket {
            blocks: vec![Block::BlockBorrowed(block.clone())],
            payload: None,
        })
        .collect();

    let stream_filename = bin_stream_log_file(payload);
    let _stream_guard = TempFileGuard::new(&stream_filename);
    tests::stream::create_file(payload, &packets, cfg.records, &stream_filename)?;
    tests::stream::read_file_borrowed_referred(payload, &stream_filename, expected_count)?;

    let protobuf_filename = protobuf_log_file(payload);
    let _protobuf_guard = TempFileGuard::new(&protobuf_filename);
    tests::protobuf::create_file(
        payload,
        packets.clone(),
        cfg.records,
        &protobuf_filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::protobuf::read_file_borrowed(payload, &protobuf_filename, expected_count)?;

    let flatbuffers_filename = flatbuffers_log_file(payload);
    let _flatbuffers_guard = TempFileGuard::new(&flatbuffers_filename);
    tests::flatbuffers::create_file(
        payload,
        packets.clone(),
        cfg.records,
        &flatbuffers_filename,
        cfg.session_reuse_limit,
        cfg.decrypt_cache_limit,
    )?;
    tests::flatbuffers::read_file_borrowed(payload, &flatbuffers_filename, expected_count)?;

    let storage_filename = bin_log_file(payload);
    let _storage_guard = TempFileGuard::new(&storage_filename);
    tests::storage::create_file(payload, packets.clone(), cfg.records, &storage_filename)?;
    tests::storage::read_file_borrowed(payload, &storage_filename, expected_count)?;

    Ok(())
}

#[test]
fn test() {
    let cfg = TestConfiguration::new();
    for _ in 0..cfg.iterations {
        assert!(runner(gen_n::<Record>(cfg.packages), &cfg).is_ok());
    }
    for _ in 0..cfg.iterations {
        assert!(runner(gen_n::<RecordBincode>(cfg.packages), &cfg).is_ok());
    }
    for _ in 0..cfg.iterations_crypt {
        assert!(runner(gen_n::<RecordCrypt>(cfg.packages), &cfg).is_ok());
    }
    for _ in 0..cfg.iterations {
        assert!(run_borrowed_logs(&gen_n::<BlockBorrowed>(cfg.packages), &cfg).is_ok());
    }
    if let Some(path) = cfg.measurements_cvs.as_deref() {
        assert!(report::write_csv(path).is_ok());
    }
}
