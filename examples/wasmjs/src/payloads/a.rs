use brec::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PayloadA {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_i8: i8,
    pub field_i16: i16,
    pub field_i32: i32,
    pub field_i64: i64,
    pub field_i128: i128,
    pub field_f32: f32,
    pub field_f64: f64,
    pub field_bool: bool,
    pub field_str: String,
    pub vec_u8: Vec<u8>,
    pub vec_u16: Vec<u16>,
    pub vec_u32: Vec<u32>,
    pub vec_u64: Vec<u64>,
    pub vec_u128: Vec<u128>,
    pub vec_i8: Vec<i8>,
    pub vec_i16: Vec<i16>,
    pub vec_i32: Vec<i32>,
    pub vec_i64: Vec<i64>,
    pub vec_i128: Vec<i128>,
    pub vec_str: Vec<String>,
}
