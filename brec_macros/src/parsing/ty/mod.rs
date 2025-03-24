use crate::*;
use std::convert::{TryFrom, TryInto};
use syn::{Expr, Ident, Type, TypeArray, TypePath};

impl TryFrom<&Type> for Ty {
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

impl TryFrom<&TypePath> for Ty {
    type Error = syn::Error;
    fn try_from(ty: &TypePath) -> Result<Self, Self::Error> {
        if let Some(ident) = ty.path.get_ident() {
            ident.try_into()
        } else {
            Err(syn::Error::new_spanned(ty, E::UnsupportedType))
        }
    }
}

impl TryFrom<&Ident> for Ty {
    type Error = syn::Error;
    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Ok(match ident.to_string().as_str() {
            "u8" => Ty::U8,
            "u16" => Ty::U16,
            "u32" => Ty::U32,
            "u64" => Ty::U64,
            "u128" => Ty::U128,
            "i8" => Ty::I8,
            "i16" => Ty::I16,
            "i32" => Ty::I32,
            "i64" => Ty::I64,
            "i128" => Ty::I128,
            "f32" => Ty::F32,
            "f64" => Ty::F64,
            "bool" => Ty::Bool,
            _linked => Ty::LinkedToU8(ident.to_string()),
        })
    }
}

impl TryFrom<&TypeArray> for Ty {
    type Error = syn::Error;
    fn try_from(ty: &TypeArray) -> Result<Self, Self::Error> {
        fn extract_array_len(len: &Expr) -> Result<usize, syn::Error> {
            if let Expr::Lit(expr_lit) = len {
                if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                    return lit_int.base10_parse::<usize>();
                }
            }
            Err(syn::Error::new_spanned(len, E::MissedArraySize))
        }
        if !matches!(Ty::try_from(&*ty.elem)?, Ty::U8) {
            Err(syn::Error::new_spanned(&*ty.elem, E::UnsupportedType))
        } else {
            Ok(Ty::Blob(extract_array_len(&ty.len)?))
        }
    }
}
