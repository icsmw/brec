use std::{
    fmt::Debug,
    io::{BufReader, Cursor, Seek},
};

use brec::{block, block::*};

use rand::{
    distr::{Distribution, StandardUniform},
    rngs::ThreadRng,
    Rng,
};

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct WithEnum {
    pub level: Level,
    data: [u8; 200],
}

impl WithEnum {
    pub fn rand() -> Self {
        let mut rng = rand::rng();
        fn slice<T>(rng: &ThreadRng) -> [T; 200]
        where
            StandardUniform: Distribution<T>,
            T: Debug,
        {
            rng.clone()
                .random_iter()
                .take(200)
                .collect::<Vec<T>>()
                .try_into()
                .expect("Expected 200 elements")
        }
        Self {
            level: rng
                .random_range(0u8..4u8)
                .try_into()
                .expect("Valid LogLevel"),
            data: slice::<u8>(&rng),
        }
    }
}

#[test]
fn from_reader() {
    use brec::block::*;
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(WithEnum::rand());
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
        match WithEnum::read(&mut reader, false) {
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
    use brec::*;
    let mut origins = Vec::new();
    let mut rng = rand::rng();
    let count = rng.random_range(5..10);
    for _ in 0..count {
        origins.push(WithEnum::rand());
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
    let mut restored: Vec<WithEnum> = Vec::new();
    let mut pos: usize = 0;
    loop {
        let referred =
            WithEnumReferred::read_from_slice(&buf[pos..pos + WithEnum::ssize() as usize], true)
                .expect("Read from slice");
        restored.push(referred.into());
        pos += WithEnum::ssize() as usize;
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
        origins.push(WithEnum::rand());
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
        match WithEnum::read(&mut reader, false) {
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
        origins.push(WithEnum::rand());
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
    let mut restored: Vec<WithEnum> = Vec::new();
    let mut pos: usize = 0;
    loop {
        let referred =
            WithEnumReferred::read_from_slice(&buf[pos..pos + WithEnum::ssize() as usize], true)
                .expect("Read from slice");
        restored.push(referred.into());
        pos += WithEnum::ssize() as usize;
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
