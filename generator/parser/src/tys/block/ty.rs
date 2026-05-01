use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::fmt;

/// f16 and f128 are unstable
#[enum_ids::enum_ids(display_variant)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum BlockTy {
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

impl BlockTy {
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
    pub fn direct(&self) -> TokenStream {
        match self {
            Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::F32
            | Self::F64
            | Self::Bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { #ty }
            }
            Self::Blob(len) => {
                quote! { [u8; #len] }
            }
            Self::LinkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { #ident }
            }
        }
    }
    pub fn referenced(&self) -> TokenStream {
        match self {
            Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::F32
            | Self::F64
            | Self::Bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { &'a  #ty }
            }
            Self::Blob(len) => {
                quote! { &'a [u8; #len] }
            }
            Self::LinkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { &'a #ident }
            }
        }
    }
}

impl fmt::Display for BlockTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::U8 => BlockTyId::U8.to_string().to_ascii_lowercase(),
                Self::U16 => BlockTyId::U16.to_string().to_ascii_lowercase(),
                Self::U32 => BlockTyId::U32.to_string().to_ascii_lowercase(),
                Self::U64 => BlockTyId::U64.to_string().to_ascii_lowercase(),
                Self::U128 => BlockTyId::U128.to_string().to_ascii_lowercase(),
                Self::I8 => BlockTyId::I8.to_string().to_ascii_lowercase(),
                Self::I16 => BlockTyId::I16.to_string().to_ascii_lowercase(),
                Self::I32 => BlockTyId::I32.to_string().to_ascii_lowercase(),
                Self::I64 => BlockTyId::I64.to_string().to_ascii_lowercase(),
                Self::I128 => BlockTyId::I128.to_string().to_ascii_lowercase(),
                Self::F32 => BlockTyId::F32.to_string().to_ascii_lowercase(),
                Self::F64 => BlockTyId::F64.to_string().to_ascii_lowercase(),
                Self::Bool => BlockTyId::Bool.to_string().to_ascii_lowercase(),
                Self::Blob(len) => {
                    let _ = BlockTyId::Blob.to_string();
                    format!("[u8;{len}]")
                }
                Self::LinkedToU8(ident) => {
                    let _ = BlockTyId::LinkedToU8.to_string();
                    ident.to_string()
                }
            }
        )
    }
}
