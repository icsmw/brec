use brec::*;

mod extended;

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
    field_u8_slice: [u8; 100],
    field_u16_slice: [u16; 100],
    field_u32_slice: [u32; 100],
    field_u64_slice: [u64; 100],
    field_u128_slice: [u128; 100],
    field_i8_slice: [i8; 100],
    field_i16_slice: [i16; 100],
    field_i32_slice: [i32; 100],
    field_i64_slice: [i64; 100],
    field_i128_slice: [i128; 100],
    field_f32_slice: [f32; 100],
    field_f64_slice: [f64; 100],
    field_bool_slice: [bool; 100],
}

// include_generated!();

fn main() {
    println!("Hello, world!");
}
