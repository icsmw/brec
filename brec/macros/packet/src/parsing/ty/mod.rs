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
            "u8" => Ty::u8,
            "u16" => Ty::u16,
            "u32" => Ty::u32,
            "u64" => Ty::u64,
            "u128" => Ty::u128,
            "i8" => Ty::i8,
            "i16" => Ty::i16,
            "i32" => Ty::i32,
            "i64" => Ty::i64,
            "i128" => Ty::i128,
            "f32" => Ty::f32,
            "f64" => Ty::f64,
            "bool" => Ty::bool,
            _linked => Ty::linkedToU8(ident.to_string()),
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
        if !matches!(Ty::try_from(&*ty.elem)?, Ty::u8) {
            Err(syn::Error::new_spanned(&*ty.elem, E::UnsupportedType))
        } else {
            Ok(Ty::blob(extract_array_len(&ty.len)?))
        }
    }
}
