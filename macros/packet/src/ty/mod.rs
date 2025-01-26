#[cfg(test)]
mod tests;

use std::convert::TryFrom;
use syn::{Expr, Ident, Type, TypeArray};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct Ty {
    pub referred: bool,
    pub def: TyDef,
}

impl Ty {
    pub fn new(def: TyDef, referred: bool) -> Self {
        Self { referred, def }
    }
}

impl TryFrom<&Type> for Ty {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => Ok(Self::new(extract_primitive(ty)?, false)),
            Type::Array(ty) => Ok(Self::new(extract_array(ty)?, false)),
            Type::Reference(ty_ref) => match ty_ref.elem.as_ref() {
                Type::Path(ty) => Ok(Self::new(extract_primitive(ty)?, true)),
                Type::Array(ty) => Ok(Self::new(extract_array(ty)?, true)),
                _ => Err(syn::Error::new_spanned(ty_ref, E::UnsupportedType)),
            },
            _ => Err(syn::Error::new_spanned(ty, E::UnsupportedType)),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum TyDef {
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
    f16,
    f32,
    f64,
    bool,
    Slice(usize, Box<Ty>),
    Option(Box<Ty>),
}

impl TryFrom<&Type> for TyDef {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => extract_primitive(ty),
            Type::Array(ty) => extract_array(ty),
            Type::Reference(ty_ref) => match ty_ref.elem.as_ref() {
                Type::Path(ty) => extract_primitive(ty),
                Type::Array(ty) => extract_array(ty),
                _ => Err(syn::Error::new_spanned(ty_ref, E::UnsupportedType)),
            },
            _ => Err(syn::Error::new_spanned(ty, E::UnsupportedType)),
        }
    }
}

fn extract_primitive(ty: &TypePath) -> Result<TyDef, syn::Error> {
    if let Some(ident) = ty.path.get_ident() {
        extract_primitive_from_ident(ident)
    } else {
        extract_option(ty)
    }
}

fn extract_primitive_from_ident(ident: &Ident) -> Result<TyDef, syn::Error> {
    Ok(match ident.to_string().as_str() {
        "u8" => TyDef::u8,
        "u16" => TyDef::u16,
        "u32" => TyDef::u32,
        "u64" => TyDef::u64,
        "u128" => TyDef::u128,
        "i8" => TyDef::i8,
        "i16" => TyDef::i16,
        "i32" => TyDef::i32,
        "i64" => TyDef::i64,
        "i128" => TyDef::i128,
        "f16" => TyDef::f16,
        "f32" => TyDef::f32,
        "f64" => TyDef::f64,
        "bool" => TyDef::bool,
        unsupported => Err(syn::Error::new_spanned(
            ident,
            E::UnsupportedFieldType(unsupported.to_owned()),
        ))?,
    })
}

fn extract_option(ty: &TypePath) -> Result<TyDef, syn::Error> {
    let segments: Vec<_> = ty.path.segments.iter().collect();
    let path = segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::");
    if ["Option", "std::option::Option"].contains(&path.as_str()) {
        let PathArguments::AngleBracketed(args) = &segments[segments.len() - 1].arguments else {
            return Err(syn::Error::new_spanned(ty, E::FailParseGenericArg));
        };
        if args.args.len() != 1 {
            return Err(syn::Error::new_spanned(ty, E::OnlySingleGenericArg));
        }
        let syn::GenericArgument::Type(inner) = &args.args[0] else {
            return Err(syn::Error::new_spanned(ty, E::FailParseGenericArg));
        };
        Ty::try_from(inner).map(|inner| TyDef::Option(Box::new(inner)))
    } else {
        Err(syn::Error::new_spanned(ty, E::UnsupportedType))
    }
}

fn extract_array(ty: &TypeArray) -> Result<TyDef, syn::Error> {
    Ok(TyDef::Slice(
        extract_array_len(&ty.len)?,
        Box::new(Ty::try_from(&*ty.elem)?),
    ))
}

fn extract_array_len(len: &Expr) -> Result<usize, syn::Error> {
    if let Expr::Lit(expr_lit) = len {
        if let syn::Lit::Int(lit_int) = &expr_lit.lit {
            return lit_int.base10_parse::<usize>();
        }
    }
    Err(syn::Error::new_spanned(len, E::MissedArraySize))
}
