mod error;
mod packet;

use std::collections::BTreeMap;

pub use error::*;
pub use packet::*;

/// Value ABI used for Rust <-> C# conversion without JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpValue {
    Null,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    /// Stores `f32` as raw IEEE-754 bits to keep lossless conversion semantics.
    F32Bits(u32),
    /// Stores `f64` as raw IEEE-754 bits to keep lossless conversion semantics.
    F64Bits(u64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<CSharpValue>),
    Object(CSharpObjectMap),
}

pub type CSharpObjectMap = BTreeMap<String, CSharpValue>;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Clone, Copy, Debug)]
pub enum CSharpFieldHint {
    Bool,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    I64,
    U64,
    I128,
    U128,
    String,
    F64,
    Vec,
    Option,
    Blob,
    Blocks,
    Payload,
    Object,
}

/// Runtime conversion helper used by generated code.
pub trait FromCSharpValue: Sized {
    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError>;
}

impl FromCSharpValue for CSharpValue {
    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        Ok(value)
    }
}

impl FromCSharpValue for CSharpObjectMap {
    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Object(obj) => Ok(obj),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::Object,
                format!("expected object, got {other:?}"),
            )),
        }
    }
}

impl FromCSharpValue for Vec<CSharpValue> {
    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Array(arr) => Ok(arr),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::Vec,
                format!("expected array, got {other:?}"),
            )),
        }
    }
}

#[inline]
pub fn from_value<T: FromCSharpValue>(
    hint: CSharpFieldHint,
    value: CSharpValue,
) -> Result<T, CSharpError> {
    T::from_csharp_value(value).map_err(|err| CSharpError::invalid_field(hint, err))
}

#[inline]
pub fn from_value_name<T: FromCSharpValue>(
    name: impl Into<String>,
    value: CSharpValue,
) -> Result<T, CSharpError> {
    T::from_csharp_value(value).map_err(|err| CSharpError::invalid_field_name(name, err))
}

#[inline]
pub fn new_object() -> CSharpObjectMap {
    CSharpObjectMap::new()
}

#[inline]
pub fn map_put(
    map: &mut CSharpObjectMap,
    key: &str,
    value: CSharpValue,
) -> Result<(), CSharpError> {
    map.insert(key.to_owned(), value);
    Ok(())
}

#[inline]
pub fn map_take(map: &mut CSharpObjectMap, key: &str) -> Result<CSharpValue, CSharpError> {
    map.remove(key)
        .ok_or_else(|| CSharpError::MissingField(key.to_owned()))
}

#[inline]
pub fn map_get(map: &CSharpObjectMap, key: &str) -> Result<CSharpValue, CSharpError> {
    map.get(key)
        .cloned()
        .ok_or_else(|| CSharpError::MissingField(key.to_owned()))
}

#[inline]
pub fn map_has(map: &CSharpObjectMap, key: &str) -> Result<bool, CSharpError> {
    Ok(map.contains_key(key))
}

#[inline]
pub fn map_keys_len_and_first(
    map: &CSharpObjectMap,
) -> Result<(usize, Option<String>), CSharpError> {
    Ok((map.len(), map.keys().next().cloned()))
}

#[inline]
pub fn new_array(cap: usize) -> Vec<CSharpValue> {
    Vec::with_capacity(cap)
}

#[inline]
pub fn list_add(list: &mut Vec<CSharpValue>, value: CSharpValue) -> Result<(), CSharpError> {
    list.push(value);
    Ok(())
}

#[inline]
pub fn list_get(list: &[CSharpValue], idx: usize) -> Result<CSharpValue, CSharpError> {
    list.get(idx).cloned().ok_or_else(|| {
        CSharpError::invalid_field(
            CSharpFieldHint::Vec,
            format!("missing element at index {idx}"),
        )
    })
}

#[inline]
pub fn list_size(list: &[CSharpValue]) -> Result<usize, CSharpError> {
    Ok(list.len())
}
