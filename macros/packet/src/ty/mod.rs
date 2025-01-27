mod referred;
mod statics;
#[cfg(test)]
mod tests;
use crate::*;
use std::{convert::TryFrom, fmt};
use syn::{Expr, Ident, Type, TypeArray};

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
    pub fn try_as_primitive(ty: &TypePath) -> Result<Ty, syn::Error> {
        if let Some(ident) = ty.path.get_ident() {
            Self::try_as_primitive_from_ident(ident)
        } else {
            Self::try_as_option(ty)
        }
    }
    pub fn try_as_primitive_from_ident(ident: &Ident) -> Result<Ty, syn::Error> {
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
            unsupported => Err(syn::Error::new_spanned(
                ident,
                E::UnsupportedFieldType(unsupported.to_owned()),
            ))?,
        })
    }
    pub fn try_as_option(ty: &TypePath) -> Result<Ty, syn::Error> {
        let segments: Vec<_> = ty.path.segments.iter().collect();
        let path = segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<String>>()
            .join("::");
        if ["Option", "std::option::Option"].contains(&path.as_str()) {
            let PathArguments::AngleBracketed(args) = &segments[segments.len() - 1].arguments
            else {
                return Err(syn::Error::new_spanned(ty, E::FailParseGenericArg));
            };
            if args.args.len() != 1 {
                return Err(syn::Error::new_spanned(ty, E::OnlySingleGenericArg));
            }
            let syn::GenericArgument::Type(inner) = &args.args[0] else {
                return Err(syn::Error::new_spanned(ty, E::FailParseGenericArg));
            };
            Ty::try_from(inner).map(|inner| Ty::Option(Box::new(inner)))
        } else {
            Err(syn::Error::new_spanned(ty, E::UnsupportedType))
        }
    }
    pub fn try_as_array(ty: &TypeArray) -> Result<Ty, syn::Error> {
        Ok(Ty::Slice(
            extract_array_len(&ty.len)?,
            Box::new(Ty::try_from(&*ty.elem)?),
        ))
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

impl TryFrom<&Type> for Ty {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => Ty::try_as_primitive(ty),
            Type::Array(ty) => Ty::try_as_array(ty),
            Type::Reference(ty) => Err(syn::Error::new_spanned(ty, E::ReferenceUnsupported)),
            _ => Err(syn::Error::new_spanned(ty, E::UnsupportedType)),
        }
    }
}

fn extract_array_len(len: &Expr) -> Result<usize, syn::Error> {
    if let Expr::Lit(expr_lit) = len {
        if let syn::Lit::Int(lit_int) = &expr_lit.lit {
            return lit_int.base10_parse::<usize>();
        }
    }
    Err(syn::Error::new_spanned(len, E::MissedArraySize))
}
