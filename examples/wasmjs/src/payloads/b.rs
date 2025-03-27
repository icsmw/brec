use brec::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct PayloadB {
    pub field_u8: Option<u8>,
    pub field_u16: Option<u16>,
    pub field_u32: Option<u32>,
    pub field_u64: Option<u64>,
    pub field_u128: Option<u128>,
    pub field_i8: Option<i8>,
    pub field_i16: Option<i16>,
    pub field_i32: Option<i32>,
    pub field_i64: Option<i64>,
    pub field_i128: Option<i128>,
    pub field_f32: Option<f32>,
    pub field_f64: Option<f64>,
    pub field_bool: Option<bool>,
    pub field_str: Option<String>,
    pub vec_u8: Option<Vec<u8>>,
    pub vec_u16: Option<Vec<u16>>,
    pub vec_u32: Option<Vec<u32>>,
    pub vec_u64: Option<Vec<u64>>,
    pub vec_u128: Option<Vec<u128>>,
    pub vec_i8: Option<Vec<i8>>,
    pub vec_i16: Option<Vec<i16>>,
    pub vec_i32: Option<Vec<i32>>,
    pub vec_i64: Option<Vec<i64>>,
    pub vec_i128: Option<Vec<i128>>,
    pub vec_str: Option<Vec<String>>,
}
