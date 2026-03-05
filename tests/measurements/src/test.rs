use crate::*;
use proptest::{prelude::*, strategy::ValueTree};
use std::path::PathBuf;

pub const MATCH: &str = "-match-";

brec::generate!();

struct TestConfiguration {
    pub iterations: usize,
    pub packages: usize,
    pub records: usize,
}

impl Default for TestConfiguration {
    fn default() -> Self {
        Self {
            iterations: 10,
            packages: 100,
            records: 1000,
        }
    }
}

impl TestConfiguration {
    pub fn new() -> Self {
        const ITERATIONS: &str = "BREC_TEST_MEASUREMENTS_ITERATIONS";
        const PACKAGES: &str = "BREC_TEST_MEASUREMENTS_PACKAGES";
        const RECORDS: &str = "BREC_TEST_MEASUREMENTS_RECORDS";
        let defaults = Self::default();
        Self {
            iterations: std::env::var(ITERATIONS)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.iterations))
                .unwrap_or(defaults.iterations),
            packages: std::env::var(PACKAGES)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.packages))
                .unwrap_or(defaults.packages),
            records: std::env::var(RECORDS)
                .map(|v| v.parse::<usize>().unwrap_or(defaults.records))
                .unwrap_or(defaults.records),
        }
    }
}

#[derive(Debug)]
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
    tests::text::create_file(rows, cfg.records, &filename)?;
    tests::text::read_file(payload, &filename)?;
    tests::text::filter_file(payload, &filename)?;
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
    tests::json::create_file(rows, cfg.records, &filename)?;
    tests::json::read_file::<R::JsonPayload>(payload, &filename)?;
    tests::json::filter_file::<R::JsonPayload>(payload, &filename)?;
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
    tests::storage::create_file(rows, cfg.records, &filename)?;
    tests::storage::read_file(payload, &filename)?;
    tests::storage::filter_file(payload, &filename)?;
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
    tests::stream::create_file(&rows, cfg.records, &filename)?;
    tests::stream::read_file(payload, &filename)?;
    tests::stream::filter_file(payload, &filename)?;
    Ok(())
}

fn run_bin_logs_storage_streamed<R>(rows: &[R], cfg: &TestConfiguration) -> std::io::Result<()>
where
    R: MeasurementRecord,
{
    let payload = R::PAYLOAD;
    let rows = rows.iter().map(|row| row.into_packet()).collect();
    let filename = bin_log_file(payload);
    let _guard = TempFileGuard::new(&filename);
    tests::streamed_storage::create_file(rows, cfg.records, &filename)?;
    tests::streamed_storage::read_file(payload, &filename)?;
    tests::streamed_storage::filter_file(payload, &filename)?;
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
    run_bin_logs::<T>(&rows, cfg)?;
    run_bin_stream_logs::<T>(&rows, cfg)?;
    run_bin_logs_storage_streamed::<T>(&rows, cfg)?;
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
}
