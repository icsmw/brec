use std::fmt;

/// f16 and f128 are unstable
#[enum_ids::enum_ids(display_variant)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Ty {
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    bool,
    Slice(usize, Box<Ty>),
    Option(Box<Ty>),
}

impl Ty {
    pub fn size(&self) -> usize {
        match self {
            Self::u8 => std::mem::size_of::<u8>(),
            Self::u16 => std::mem::size_of::<u16>(),
            Self::u32 => std::mem::size_of::<u32>(),
            Self::u64 => std::mem::size_of::<u64>(),
            Self::u128 => std::mem::size_of::<u128>(),
            Self::i8 => std::mem::size_of::<i8>(),
            Self::i16 => std::mem::size_of::<i16>(),
            Self::i32 => std::mem::size_of::<i32>(),
            Self::i64 => std::mem::size_of::<i64>(),
            Self::i128 => std::mem::size_of::<i128>(),
            Self::f32 => std::mem::size_of::<f32>(),
            Self::f64 => std::mem::size_of::<f64>(),
            Self::bool => std::mem::size_of::<bool>(),
            Self::Slice(len, ty) => len * ty.size(),
            Self::Option(ty) => ty.size() + 1,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::u8 => TyId::u8.to_string(),
                Self::u16 => TyId::u16.to_string(),
                Self::u32 => TyId::u32.to_string(),
                Self::u64 => TyId::u64.to_string(),
                Self::u128 => TyId::u128.to_string(),
                Self::i8 => TyId::i8.to_string(),
                Self::i16 => TyId::i16.to_string(),
                Self::i32 => TyId::i32.to_string(),
                Self::i64 => TyId::i64.to_string(),
                Self::i128 => TyId::i128.to_string(),
                Self::f32 => TyId::f32.to_string(),
                Self::f64 => TyId::f64.to_string(),
                Self::bool => TyId::bool.to_string(),
                Self::Slice(len, ty) => format!("[{ty};{len}]"),
                Self::Option(ty) => format!("Option<{ty}>"),
            }
        )
    }
}
