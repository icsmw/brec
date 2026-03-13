use brec::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCA {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCB {
    pub field_i8: i8,
    pub field_i16: i16,
    pub field_i32: i32,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCC {
    pub field_bool: bool,
    pub field_str: String,
    pub vec_u8: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum NestedEnumC {
    One(String),
    Two(Vec<u8>),
    Three,
    Four(NestedStructCC),
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct PayloadC {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_struct_a: NestedStructCA,
    pub field_struct_b: Option<NestedStructCB>,
    pub field_struct_c: Vec<NestedStructCC>,
    pub field_enum: NestedEnumC,
    pub vec_enum: Vec<NestedEnumC>,
}
