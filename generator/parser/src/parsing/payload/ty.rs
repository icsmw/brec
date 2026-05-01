use crate::parsing::block::ty::extract_array_len;
use crate::*;
use quote::ToTokens;
use std::convert::{TryFrom, TryInto};
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, PathArguments, PathSegment, Type,
    TypeArray, TypePath, TypeTuple,
};

impl TryFrom<&Type> for PayloadTy {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => ty.try_into(),
            Type::Array(ty) => ty.try_into(),
            Type::Tuple(ty) => ty.try_into(),
            Type::Reference(ty) => Err(syn::Error::new_spanned(ty, E::ReferenceUnsupported)),
            _ => Ok(PayloadTy::Struct(ty.to_token_stream().to_string())),
        }
    }
}

impl TryFrom<&TypePath> for PayloadTy {
    type Error = syn::Error;

    fn try_from(ty: &TypePath) -> Result<Self, Self::Error> {
        if let Some(ident) = ty.path.get_ident()
            && let Ok(primitive) = PayloadTy::try_from(ident)
        {
            return Ok(primitive);
        }

        let Some(segment) = ty.path.segments.last() else {
            return Err(syn::Error::new_spanned(ty, E::UnsupportedType));
        };
        match segment.ident.to_string().as_str() {
            "Vec" => Ok(PayloadTy::Vec(Box::new(extract_one_generic(segment, ty)?))),
            "Option" => Ok(PayloadTy::Option(Box::new(extract_one_generic(
                segment, ty,
            )?))),
            "HashSet" => Ok(PayloadTy::HashSet(Box::new(extract_one_generic(
                segment, ty,
            )?))),
            "BTreeSet" => Ok(PayloadTy::BTreeSet(Box::new(extract_one_generic(
                segment, ty,
            )?))),
            "HashMap" => {
                let (key, value) = extract_two_generics(segment, ty)?;
                Ok(PayloadTy::HashMap(Box::new(key), Box::new(value)))
            }
            "BTreeMap" => {
                let (key, value) = extract_two_generics(segment, ty)?;
                Ok(PayloadTy::BTreeMap(Box::new(key), Box::new(value)))
            }
            _ => Ok(PayloadTy::Struct(ty.to_token_stream().to_string())),
        }
    }
}

impl TryFrom<&Ident> for PayloadTy {
    type Error = syn::Error;

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        match ident.to_string().as_str() {
            "String" => Ok(PayloadTy::String),
            "u8" => Ok(PayloadTy::U8),
            "u16" => Ok(PayloadTy::U16),
            "u32" => Ok(PayloadTy::U32),
            "u64" => Ok(PayloadTy::U64),
            "u128" => Ok(PayloadTy::U128),
            "i8" => Ok(PayloadTy::I8),
            "i16" => Ok(PayloadTy::I16),
            "i32" => Ok(PayloadTy::I32),
            "i64" => Ok(PayloadTy::I64),
            "i128" => Ok(PayloadTy::I128),
            "f32" => Ok(PayloadTy::F32),
            "f64" => Ok(PayloadTy::F64),
            "bool" => Ok(PayloadTy::Bool),
            _ => Err(syn::Error::new_spanned(ident, E::UnsupportedType)),
        }
    }
}

impl TryFrom<&TypeArray> for PayloadTy {
    type Error = syn::Error;

    fn try_from(ty: &TypeArray) -> Result<Self, Self::Error> {
        let len = extract_array_len(&ty.len)?;
        let inner = PayloadTy::try_from(&*ty.elem)?;
        if matches!(inner, PayloadTy::U8) {
            Ok(PayloadTy::Blob(len))
        } else {
            Ok(PayloadTy::Array(Box::new(inner), len))
        }
    }
}

impl TryFrom<&TypeTuple> for PayloadTy {
    type Error = syn::Error;

    fn try_from(tuple: &TypeTuple) -> Result<Self, Self::Error> {
        Ok(PayloadTy::Tuple(
            tuple
                .elems
                .iter()
                .map(PayloadTy::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

fn extract_one_generic(segment: &PathSegment, source: &TypePath) -> Result<PayloadTy, syn::Error> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        return Err(syn::Error::new_spanned(source, E::UnsupportedType));
    };
    let args = args.iter().collect::<Vec<_>>();
    let [arg] = args.as_slice() else {
        return Err(syn::Error::new_spanned(source, E::UnsupportedType));
    };
    let GenericArgument::Type(ty) = arg else {
        return Err(syn::Error::new_spanned(arg, E::UnsupportedType));
    };
    PayloadTy::try_from(ty)
}

fn extract_two_generics(
    segment: &PathSegment,
    source: &TypePath,
) -> Result<(PayloadTy, PayloadTy), syn::Error> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        return Err(syn::Error::new_spanned(source, E::UnsupportedType));
    };
    let args = args.iter().collect::<Vec<_>>();
    let [key, value] = args.as_slice() else {
        return Err(syn::Error::new_spanned(source, E::UnsupportedType));
    };
    let GenericArgument::Type(key) = key else {
        return Err(syn::Error::new_spanned(key, E::UnsupportedType));
    };
    let GenericArgument::Type(value) = value else {
        return Err(syn::Error::new_spanned(value, E::UnsupportedType));
    };
    Ok((PayloadTy::try_from(key)?, PayloadTy::try_from(value)?))
}
