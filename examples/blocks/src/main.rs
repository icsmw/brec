#[cfg(test)]
mod block_blob;
// #[cfg(test)]
// mod block_blobs_max;
// #[cfg(test)]
// mod block_bool;
// #[cfg(test)]
// mod block_comb;
// #[cfg(test)]
// mod block_f32;
// #[cfg(test)]
// mod block_f64;
// #[cfg(test)]
// mod block_i128;
// #[cfg(test)]
// mod block_i16;
// #[cfg(test)]
// mod block_i32;
// #[cfg(test)]
// mod block_i64;
// #[cfg(test)]
// mod block_i8;
// #[cfg(test)]
// mod block_u128;
// #[cfg(test)]
// mod block_u16;
// #[cfg(test)]
// mod block_u32;
// #[cfg(test)]
// mod block_u64;
// #[cfg(test)]
// mod block_u8;

#[cfg(test)]
mod test;

// use std::{
//     fmt::Debug,
//     io::{BufReader, Cursor, Seek},
// };

// use brec::prelude::*;

// use rand::{
//     distr::{Distribution, StandardUniform},
//     rngs::ThreadRng,
//     Rng,
// };

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockU8 {
//     field_u8: u8,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockU16 {
//     field_u16: u16,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockU32 {
//     field_u32: u32,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockU64 {
//     field_u64: u64,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockU128 {
//     field_u128: u128,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockI8 {
//     field_i8: i8,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockI16 {
//     field_i16: i16,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockI32 {
//     field_i32: i32,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockI64 {
//     field_i64: i64,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockI128 {
//     field_i128: i128,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockF32 {
//     field_f32: f32,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockF64 {
//     field_f64: f64,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockBool {
//     field_bool: bool,
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockSlice {
//     field_slice: [u8; 100],
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct BlockSlices {
//     field_slice_a: [u8; 1],             // 1B
//     field_slice_b: [u8; 1_024],         // 1Kb
//     field_slice_c: [u8; 1_048_576],     // 1Mb
//     field_slice_d: [u8; 1_073_741_824], // 1Gb
// }

// #[derive(Debug, PartialEq, Clone)]
// #[block]
// pub struct CustomBlock {
//     field_u8: u8,
//     field_u16: u16,
//     field_u32: u32,
//     field_u64: u64,
//     field_u128: u128,
//     field_i8: i8,
//     field_i16: i16,
//     field_i32: i32,
//     field_i64: i64,
//     field_i128: i128,
//     field_f32: f32,
//     field_f64: f64,
//     field_bool: bool,
//     blob_a: [u8; 1],
//     blob_b: [u8; 100],
//     blob_c: [u8; 1000],
//     blob_d: [u8; 10000],
// }

// impl CustomBlock {
//     pub fn rand() -> Self {
//         let mut rng = rand::rng();
//         fn slice<T>(rng: &ThreadRng) -> [T; 100]
//         where
//             StandardUniform: Distribution<T>,
//             T: Debug,
//         {
//             rng.clone()
//                 .random_iter()
//                 .take(100)
//                 .collect::<Vec<T>>()
//                 .try_into()
//                 .expect("Expected 100 elements")
//         }
//         Self {
//             field_u8: rng.random(),
//             field_u16: rng.random(),
//             field_u32: rng.random(),
//             field_u64: rng.random(),
//             field_u128: rng.random(),
//             field_i8: rng.random(),
//             field_i16: rng.random(),
//             field_i32: rng.random(),
//             field_i64: rng.random(),
//             field_i128: rng.random(),
//             field_f32: rng.random(),
//             field_f64: rng.random(),
//             field_bool: rng.random_bool(1.0 / 3.0),
//             blob_a: slice::<u8>(&rng),
//             blob_b: slice::<u8>(&rng),
//         }
//     }
// }

// #[test]
// fn from_reader() {
//     let mut origins = Vec::new();
//     let mut rng = rand::rng();
//     let count = rng.random_range(5..10);
//     for _ in 0..count {
//         origins.push(CustomBlock::rand());
//     }
//     let mut buf: Vec<u8> = Vec::new();
//     for blk in origins.iter() {
//         println!(
//             "write: {} bytes",
//             WriteTo::write(blk, &mut buf).expect("Block is written")
//         );
//     }
//     let size = buf.len() as u64;
//     println!("created: {count}; total size: {size}");
//     let mut restored = Vec::new();
//     let mut reader = BufReader::new(Cursor::new(buf));
//     let mut consumed = 0;
//     loop {
//         match CustomBlock::read(&mut reader, false) {
//             Ok(blk) => {
//                 consumed = reader.stream_position().expect("Position is read");
//                 restored.push(blk);
//             }
//             Err(err) => {
//                 println!("{err}");
//                 break;
//             }
//         }
//     }
//     assert_eq!(size, consumed);
//     assert_eq!(origins.len(), restored.len());
//     for (left, right) in restored.iter().zip(origins.iter()) {
//         assert_eq!(left, right);
//     }
// }

// #[test]
// fn from_slice() {
//     let mut origins = Vec::new();
//     let mut rng = rand::rng();
//     let count = rng.random_range(5..10);
//     for _ in 0..count {
//         origins.push(CustomBlock::rand());
//     }
//     let mut buf: Vec<u8> = Vec::new();
//     for blk in origins.iter() {
//         println!(
//             "write: {} bytes",
//             WriteTo::write(blk, &mut buf).expect("Block is written")
//         );
//     }
//     let size = buf.len() as u64;
//     println!("created: {count}; total size: {size}");
//     let mut restored: Vec<CustomBlock> = Vec::new();
//     let mut pos: usize = 0;
//     loop {
//         let referred = CustomBlockReferred::read_from_slice(
//             &buf[pos..pos + CustomBlock::ssize() as usize],
//             true,
//         )
//         .expect("Read from slice");
//         restored.push(referred.into());
//         pos += CustomBlock::ssize() as usize;
//         println!("read bytes: {pos}; blocks: {}", restored.len());
//         if restored.len() == origins.len() {
//             break;
//         }
//     }
//     assert_eq!(origins.len(), restored.len());
//     for (left, right) in restored.iter().zip(origins.iter()) {
//         assert_eq!(left, right);
//     }
// }

// #[test]
// fn from_vectored_reader() {
//     let mut origins = Vec::new();
//     let mut rng = rand::rng();
//     let count = rng.random_range(5..10);
//     for _ in 0..count {
//         origins.push(CustomBlock::rand());
//     }
//     let mut buf: Vec<u8> = Vec::new();
//     for blk in origins.iter() {
//         println!(
//             "write: {} bytes",
//             WriteVectoredTo::write_vectored(blk, &mut buf).expect("Block is written")
//         );
//     }
//     let size = buf.len() as u64;
//     println!("created: {count}; total size: {size}");
//     let mut restored = Vec::new();
//     let mut reader = BufReader::new(Cursor::new(buf));
//     let mut consumed = 0;
//     loop {
//         match CustomBlock::read(&mut reader, false) {
//             Ok(blk) => {
//                 consumed = reader.stream_position().expect("Position is read");
//                 restored.push(blk);
//             }
//             Err(err) => {
//                 println!("{err}");
//                 break;
//             }
//         }
//     }
//     assert_eq!(size, consumed);
//     assert_eq!(origins.len(), restored.len());
//     for (left, right) in restored.iter().zip(origins.iter()) {
//         assert_eq!(left, right);
//     }
// }

// #[test]
// fn from_vectored_all_reader() {
//     let mut origins = Vec::new();
//     let mut rng = rand::rng();
//     let count = rng.random_range(5..10);
//     for _ in 0..count {
//         origins.push(CustomBlock::rand());
//     }
//     let mut buf: Vec<u8> = Vec::new();
//     for blk in origins.iter() {
//         WriteVectoredTo::write_vectored_all(blk, &mut buf).expect("Block is written")
//     }
//     let size = buf.len() as u64;
//     println!("created: {count}; total size: {size}");
//     let mut restored = Vec::new();
//     let mut reader = BufReader::new(Cursor::new(buf));
//     let mut consumed = 0;
//     loop {
//         match CustomBlock::read(&mut reader, false) {
//             Ok(blk) => {
//                 consumed = reader.stream_position().expect("Position is read");
//                 restored.push(blk);
//             }
//             Err(err) => {
//                 println!("{err}");
//                 break;
//             }
//         }
//     }
//     assert_eq!(size, consumed);
//     assert_eq!(origins.len(), restored.len());
//     for (left, right) in restored.iter().zip(origins.iter()) {
//         assert_eq!(left, right);
//     }
// }

// #[test]
// fn from_vectored_slice() {
//     let mut origins = Vec::new();
//     let mut rng = rand::rng();
//     let count = rng.random_range(5..10);
//     for _ in 0..count {
//         origins.push(CustomBlock::rand());
//     }
//     let mut buf: Vec<u8> = Vec::new();
//     for blk in origins.iter() {
//         println!(
//             "write: {} bytes",
//             WriteVectoredTo::write_vectored(blk, &mut buf).expect("Block is written")
//         );
//     }
//     let size = buf.len() as u64;
//     println!("created: {count}; total size: {size}");
//     let mut restored: Vec<CustomBlock> = Vec::new();
//     let mut pos: usize = 0;
//     loop {
//         let referred = CustomBlockReferred::read_from_slice(
//             &buf[pos..pos + CustomBlock::ssize() as usize],
//             true,
//         )
//         .expect("Read from slice");
//         restored.push(referred.into());
//         pos += CustomBlock::ssize() as usize;
//         println!("read bytes: {pos}; blocks: {}", restored.len());
//         if restored.len() == origins.len() {
//             break;
//         }
//     }
//     assert_eq!(origins.len(), restored.len());
//     for (left, right) in restored.iter().zip(origins.iter()) {
//         assert_eq!(left, right);
//     }
// }
fn main() {}
