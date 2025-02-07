use std::{
    fmt::{format, Debug},
    io::{BufReader, Cursor, Seek},
    ops::Deref,
};

use brec::*;
use rand::{
    distr::{Distribution, StandardUniform},
    rngs::ThreadRng,
    Rng,
};

mod extended;

#[derive(Debug)]
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

impl TryFrom<u8> for Level {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Level::Err),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Debug),
            3 => Ok(Level::Info),
            invalid => Err(format!("{invalid} isn't valid value for Level")),
        }
    }
}

impl From<&Level> for u8 {
    fn from(value: &Level) -> Self {
        match value {
            Level::Err => 0,
            Level::Warn => 1,
            Level::Debug => 2,
            Level::Info => 3,
        }
    }
}

#[block]
pub struct WithEnum {
    pub level: Level,
    data: [u8; 250],
}

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct CustomBlock {
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    blob_a: [u8; 100],
    blob_b: [u8; 100],
}

include_generated!();

impl CustomBlock {
    pub fn rand() -> Self {
        let mut rng = rand::rng();
        fn slice<T>(rng: &ThreadRng) -> [T; 100]
        where
            StandardUniform: Distribution<T>,
            T: Debug,
        {
            rng.clone()
                .random_iter()
                .take(100)
                .collect::<Vec<T>>()
                .try_into()
                .expect("Expected 100 elements")
        }
        Self {
            field_u8: rng.random(),
            field_u16: rng.random(),
            field_u32: rng.random(),
            field_u64: rng.random(),
            field_u128: rng.random(),
            field_i8: rng.random(),
            field_i16: rng.random(),
            field_i32: rng.random(),
            field_i64: rng.random(),
            field_i128: rng.random(),
            field_f32: rng.random(),
            field_f64: rng.random(),
            field_bool: rng.random_bool(1.0 / 3.0),
            blob_a: slice::<u8>(&rng),
            blob_b: slice::<u8>(&rng),
        }
    }
}

#[test]
fn from_reader() {
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(CustomBlock::rand());
    }
    let mut buf: Vec<u8> = Vec::new();
    for blk in origins.iter() {
        println!(
            "write: {} bytes",
            blk.write(&mut buf).expect("Block is written")
        );
    }
    let size = buf.len() as u64;
    println!("created: {count}; total size: {size}");
    let mut restored = Vec::new();
    let mut reader = BufReader::new(Cursor::new(buf));
    let mut consumed = 0;
    loop {
        match CustomBlock::read(&mut reader, false) {
            Ok(blk) => {
                consumed = reader.stream_position().expect("Position is read");
                restored.push(blk);
            }
            Err(err) => {
                println!("{err}");
                break;
            }
        }
    }
    assert_eq!(size, consumed);
    assert_eq!(origins.len(), restored.len());
    for (left, right) in restored.iter().zip(origins.iter()) {
        assert_eq!(left, right);
    }
}

#[test]
fn from_slice() {
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(CustomBlock::rand());
    }
    let mut buf: Vec<u8> = Vec::new();
    for blk in origins.iter() {
        println!(
            "write: {} bytes",
            blk.write(&mut buf).expect("Block is written")
        );
    }
    let size = buf.len() as u64;
    println!("created: {count}; total size: {size}");
    let mut restored: Vec<CustomBlock> = Vec::new();
    let mut pos: usize = 0;
    loop {
        let referred = CustomBlockReferred::read_from_slice(
            &buf[pos..pos + CustomBlock::size() as usize],
            true,
        )
        .expect("Read from slice");
        restored.push(referred.into());
        pos += CustomBlock::size() as usize;
        println!("read bytes: {pos}; blocks: {}", restored.len());
        if restored.len() == origins.len() {
            break;
        }
    }
    assert_eq!(origins.len(), restored.len());
    for (left, right) in restored.iter().zip(origins.iter()) {
        assert_eq!(left, right);
    }
}

#[test]
fn from_reader_owned() {
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(CustomBlock::rand());
    }
    let mut buf: Vec<u8> = Vec::new();
    for blk in origins.iter() {
        println!(
            "write: {} bytes",
            WriteOwned::write(blk.clone(), &mut buf).expect("Block is written")
        );
    }
    let size = buf.len() as u64;
    println!("created: {count}; total size: {size}");
    let mut restored = Vec::new();
    let mut reader = BufReader::new(Cursor::new(buf));
    let mut consumed = 0;
    loop {
        match CustomBlock::read(&mut reader, false) {
            Ok(blk) => {
                consumed = reader.stream_position().expect("Position is read");
                restored.push(blk);
            }
            Err(err) => {
                println!("{err}");
                break;
            }
        }
    }
    assert_eq!(size, consumed);
    assert_eq!(origins.len(), restored.len());
    for (left, right) in restored.iter().zip(origins.iter()) {
        assert_eq!(left, right);
    }
}

#[test]
fn from_slice_owned() {
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(CustomBlock::rand());
    }
    let mut buf: Vec<u8> = Vec::new();
    for blk in origins.iter() {
        println!(
            "write: {} bytes",
            WriteOwned::write(blk.clone(), &mut buf).expect("Block is written")
        );
    }
    let size = buf.len() as u64;
    println!("created: {count}; total size: {size}");
    let mut restored: Vec<CustomBlock> = Vec::new();
    let mut pos: usize = 0;
    loop {
        let referred = CustomBlockReferred::read_from_slice(
            &buf[pos..pos + CustomBlock::size() as usize],
            true,
        )
        .expect("Read from slice");
        restored.push(referred.into());
        pos += CustomBlock::size() as usize;
        println!("read bytes: {pos}; blocks: {}", restored.len());
        if restored.len() == origins.len() {
            break;
        }
    }
    assert_eq!(origins.len(), restored.len());
    for (left, right) in restored.iter().zip(origins.iter()) {
        assert_eq!(left, right);
    }
}
fn main() {
    println!("This is just an example. No sense to run it ;)");
}
