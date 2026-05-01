use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum PayloadTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Bool,
    Blob(usize),
    Struct(String),
    Enum(String),
    HashMap(Box<PayloadTy>, Box<PayloadTy>),
    BTreeMap(Box<PayloadTy>, Box<PayloadTy>),
    HashSet(Box<PayloadTy>),
    BTreeSet(Box<PayloadTy>),
    Vec(Box<PayloadTy>),
    Option(Box<PayloadTy>),
    Tuple(Vec<PayloadTy>),
    Array(Box<PayloadTy>, usize),
}

impl fmt::Display for PayloadTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::U8 => "u8".to_owned(),
                Self::U16 => "u16".to_owned(),
                Self::U32 => "u32".to_owned(),
                Self::U64 => "u64".to_owned(),
                Self::U128 => "u128".to_owned(),
                Self::I8 => "i8".to_owned(),
                Self::I16 => "i16".to_owned(),
                Self::I32 => "i32".to_owned(),
                Self::I64 => "i64".to_owned(),
                Self::I128 => "i128".to_owned(),
                Self::F32 => "f32".to_owned(),
                Self::F64 => "f64".to_owned(),
                Self::Bool => "bool".to_owned(),
                Self::Blob(len) => format!("[u8;{len}]"),
                Self::Struct(ty) => format!("Struct<{ty}>"),
                Self::Enum(ty) => format!("Enum<{ty}>"),
                Self::HashMap(key, value) => format!("HashMap<{}, {}>", key, value),
                Self::BTreeMap(key, value) => format!("BTreeMap<{}, {}>", key, value),
                Self::HashSet(inner) => format!("HashSet<{inner}>"),
                Self::BTreeSet(inner) => format!("BTreeSet<{inner}>"),
                Self::Vec(inner) => format!("Vec<{inner}>"),
                Self::Option(inner) => format!("Option<{inner}>"),
                Self::Tuple(items) => format!(
                    "Tuple<{}>",
                    items
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Self::Array(inner, len) => format!("Array<{inner}; {len}>"),
            }
        )
    }
}
