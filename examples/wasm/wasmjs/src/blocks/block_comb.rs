use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlockCombination {
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
}
