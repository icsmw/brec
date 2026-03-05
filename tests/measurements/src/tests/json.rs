use crate::*;
use std::io::Write;
use std::{
    fs::{metadata, File},
    io::{BufRead, BufReader},
    time::Instant,
};

pub fn create_file<T>(rows: Vec<JSONRow<T>>, mut count: usize, filename: &str) -> std::io::Result<()>
where
    T: serde::Serialize,
{
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
        for row in rows.iter() {
            file.write_all(format!("{}\n", serde_json::to_string(row)?).as_bytes())?;
        }
        count -= 1;
    }
    file.flush()
}

pub fn read_file<T>(payload: report::PayloadKind, filename: &str) -> std::io::Result<()>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        let _ = serde_json::from_str::<JSONRow<T>>(&line)?;
        count += 1;
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Json,
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

pub fn filter_file<T>(payload: report::PayloadKind, filename: &str) -> std::io::Result<()>
where
    T: crate::content::MatchValue + for<'de> serde::Deserialize<'de>,
{
    let now = Instant::now();
    let metrics = crate::metrics::Tracker::start();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        let msg = serde_json::from_str::<JSONRow<T>>(&line)?;
        if matches!(msg.meta.level, Level::Err) && msg.payload.contains_match() {
            count += 1;
        }
    }
    let usage = metrics.finish();
    report::add(
        payload,
        report::Platform::Json,
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
