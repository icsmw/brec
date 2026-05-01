use crate::*;
use std::convert::{TryFrom, TryInto};
use syn::{Expr, Ident, Type, TypeArray, TypePath};

impl TryFrom<&Type> for BlockTy {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => ty.try_into(),
            Type::Array(ty) => ty.try_into(),
            Type::Reference(ty) => Err(syn::Error::new_spanned(ty, E::ReferenceUnsupported)),
            _ => Err(syn::Error::new_spanned(ty, E::UnsupportedType)),
        }
    }
}

impl TryFrom<&TypePath> for BlockTy {
    type Error = syn::Error;
    fn try_from(ty: &TypePath) -> Result<Self, Self::Error> {
        if let Some(ident) = ty.path.get_ident() {
            ident.try_into()
        } else {
            Err(syn::Error::new_spanned(ty, E::UnsupportedType))
        }
    }
}

impl TryFrom<&Ident> for BlockTy {
    type Error = syn::Error;
    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Ok(match ident.to_string().as_str() {
            "u8" => BlockTy::U8,
            "u16" => BlockTy::U16,
            "u32" => BlockTy::U32,
            "u64" => BlockTy::U64,
            "u128" => BlockTy::U128,
            "i8" => BlockTy::I8,
            "i16" => BlockTy::I16,
            "i32" => BlockTy::I32,
            "i64" => BlockTy::I64,
            "i128" => BlockTy::I128,
            "f32" => BlockTy::F32,
            "f64" => BlockTy::F64,
            "bool" => BlockTy::Bool,
            _linked => BlockTy::LinkedToU8(ident.to_string()),
        })
    }
}

impl TryFrom<&TypeArray> for BlockTy {
    type Error = syn::Error;
    fn try_from(ty: &TypeArray) -> Result<Self, Self::Error> {
        if !matches!(BlockTy::try_from(&*ty.elem)?, BlockTy::U8) {
            Err(syn::Error::new_spanned(&*ty.elem, E::UnsupportedType))
        } else {
            Ok(BlockTy::Blob(extract_array_len(&ty.len)?))
        }
    }
}

pub fn extract_array_len(len: &Expr) -> Result<usize, syn::Error> {
    if let Expr::Lit(expr_lit) = len
        && let syn::Lit::Int(lit_int) = &expr_lit.lit
    {
        return lit_int.base10_parse::<usize>();
    }
    Err(syn::Error::new_spanned(len, E::MissedArraySize))
}
