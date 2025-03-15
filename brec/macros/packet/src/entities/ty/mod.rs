use std::fmt;

/// f16 and f128 are unstable
#[enum_ids::enum_ids(display_variant)]
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Ty {
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
    LinkedToU8(String),
}

impl Ty {
    pub fn size(&self) -> usize {
        match self {
            Self::U8 => std::mem::size_of::<u8>(),
            Self::U16 => std::mem::size_of::<u16>(),
            Self::U32 => std::mem::size_of::<u32>(),
            Self::U64 => std::mem::size_of::<u64>(),
            Self::U128 => std::mem::size_of::<u128>(),
            Self::I8 => std::mem::size_of::<i8>(),
            Self::I16 => std::mem::size_of::<i16>(),
            Self::I32 => std::mem::size_of::<i32>(),
            Self::I64 => std::mem::size_of::<i64>(),
            Self::I128 => std::mem::size_of::<i128>(),
            Self::F32 => std::mem::size_of::<f32>(),
            Self::F64 => std::mem::size_of::<f64>(),
            Self::Bool => std::mem::size_of::<bool>(),
            Self::Blob(len) => *len,
            Self::LinkedToU8(..) => std::mem::size_of::<u8>(),
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::U8 => TyId::U8.to_string().to_ascii_lowercase(),
                Self::U16 => TyId::U16.to_string().to_ascii_lowercase(),
                Self::U32 => TyId::U32.to_string().to_ascii_lowercase(),
                Self::U64 => TyId::U64.to_string().to_ascii_lowercase(),
                Self::U128 => TyId::U128.to_string().to_ascii_lowercase(),
                Self::I8 => TyId::I8.to_string().to_ascii_lowercase(),
                Self::I16 => TyId::I16.to_string().to_ascii_lowercase(),
                Self::I32 => TyId::I32.to_string().to_ascii_lowercase(),
                Self::I64 => TyId::I64.to_string().to_ascii_lowercase(),
                Self::I128 => TyId::I128.to_string().to_ascii_lowercase(),
                Self::F32 => TyId::F32.to_string().to_ascii_lowercase(),
                Self::F64 => TyId::F64.to_string().to_ascii_lowercase(),
                Self::Bool => TyId::Bool.to_string().to_ascii_lowercase(),
                Self::Blob(len) => {
                    // Just to avoid rust warning "never constructed" for TyId::Blob
                    let _ = TyId::Blob.to_string();
                    format!("[u8;{len}]")
                }
                Self::LinkedToU8(ident) => {
                    // Just to avoid rust warning "never constructed" for TyId::LinkedToU8
                    let _ = TyId::LinkedToU8.to_string();
                    ident.to_string()
                }
            }
        )
    }
}
