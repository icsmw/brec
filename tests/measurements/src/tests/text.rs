use crate::test::MATCH;
use crate::*;
use std::io::Write;
use std::{
    fs::{metadata, File},
    io::{BufRead, BufReader},
    time::Instant,
};

pub fn create_file(rows: Vec<TextualRow>, mut count: usize, filename: &str) -> std::io::Result<()> {
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
            file.write_all(format!("{}\n", row.msg,).as_bytes())?;
        }
        count -= 1;
    }
    file.flush()
}

pub fn read_file(filename: &str) -> std::io::Result<()> {
    let now = Instant::now();
    let tmp = std::env::temp_dir().join(filename);
    let size = metadata(&tmp).expect("Read File Meta").len();
    let file = File::open(tmp)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line_result in reader.lines() {
        let _line = line_result?;
        count += 1;
    }
    report::add(
        report::Platform::Text,
        report::TestCase::Reading,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
        },
    );
    Ok(())
}

pub fn filter_file(filename: &str) -> std::io::Result<()> {
    let now = Instant::now();
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
    report::add(
        report::Platform::Text,
        report::TestCase::Filtering,
        report::TestResults {
            size,
            count,
            time: now.elapsed().as_millis(),
        },
    );
    Ok(())
}
