use crate::*;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use std::fmt;

impl Default for Ty {
    fn default() -> Self {
        Self::u8
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub(crate) enum TyValue {
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    f32(f32),
    f64(f64),
    bool(bool),
    blob(Vec<u8>),
}

impl TyValue {
    pub fn into_ts(&self) -> TokenStream {
        match self {
            Self::u8(v) => quote! { #v },
            Self::u16(v) => quote! { #v },
            Self::u32(v) => quote! { #v },
            Self::u64(v) => quote! { #v },
            Self::u128(v) => quote! { #v },
            Self::i8(v) => quote! { #v },
            Self::i16(v) => quote! { #v },
            Self::i32(v) => quote! { #v },
            Self::i64(v) => quote! { #v },
            Self::i128(v) => quote! { #v },
            Self::f32(v) => quote! { #v },
            Self::f64(v) => quote! { #v },
            Self::bool(v) => quote! { #v },
            Self::blob(v) => {
                let vals = v
                    .iter()
                    .map(|v| quote! { #v })
                    .collect::<Vec<TokenStream>>();
                quote! {[#(#vals,)*]}
            }
        }
    }
}

impl Default for TyValue {
    fn default() -> Self {
        Self::u8(0)
    }
}

impl fmt::Display for TyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::u8(v) => v.to_string(),
                Self::u16(v) => v.to_string(),
                Self::u32(v) => v.to_string(),
                Self::u64(v) => v.to_string(),
                Self::u128(v) => v.to_string(),
                Self::i8(v) => v.to_string(),
                Self::i16(v) => v.to_string(),
                Self::i32(v) => v.to_string(),
                Self::i64(v) => v.to_string(),
                Self::i128(v) => v.to_string(),
                Self::f32(v) => v.to_string(),
                Self::f64(v) => v.to_string(),
                Self::bool(v) => v.to_string(),
                Self::blob(v) => format!(
                    "[{};{}]",
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    v.len()
                ),
            }
        )
    }
}

impl Arbitrary for TyValue {
    type Parameters = Ty;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(ty: Ty) -> Self::Strategy {
        match ty {
            Ty::u8 => any::<u8>().prop_map(TyValue::u8).boxed(),
            Ty::u16 => any::<u16>().prop_map(TyValue::u16).boxed(),
            Ty::u32 => any::<u32>().prop_map(TyValue::u32).boxed(),
            Ty::u64 => any::<u64>().prop_map(TyValue::u64).boxed(),
            Ty::u128 => any::<u128>().prop_map(TyValue::u128).boxed(),
            Ty::i8 => any::<i8>().prop_map(TyValue::i8).boxed(),
            Ty::i16 => any::<i16>().prop_map(TyValue::i16).boxed(),
            Ty::i32 => any::<i32>().prop_map(TyValue::i32).boxed(),
            Ty::i64 => any::<i64>().prop_map(TyValue::i64).boxed(),
            Ty::i128 => any::<i128>().prop_map(TyValue::i128).boxed(),
            Ty::f32 => any::<f32>().prop_map(TyValue::f32).boxed(),
            Ty::f64 => any::<f64>().prop_map(TyValue::f64).boxed(),
            Ty::bool => any::<bool>().prop_map(TyValue::bool).boxed(),
            Ty::blob(len) => prop::collection::vec(any::<u8>(), len)
                .prop_map(|v| TyValue::blob(v.into_iter().collect()))
                .boxed(),
        }
    }
}

impl Arbitrary for Ty {
    type Parameters = bool;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(incl_blob: bool) -> Self::Strategy {
        if incl_blob {
            (
                (0usize..=1024),
                prop_oneof![
                    Just(Ty::u8),
                    Just(Ty::u16),
                    Just(Ty::u32),
                    Just(Ty::u64),
                    Just(Ty::u128),
                    Just(Ty::i8),
                    Just(Ty::i16),
                    Just(Ty::i32),
                    Just(Ty::i64),
                    Just(Ty::i128),
                    Just(Ty::f32),
                    Just(Ty::f64),
                    Just(Ty::bool),
                    Just(Ty::blob(0))
                ]
                .boxed(),
            )
                .prop_map(|(len, ty)| match ty {
                    Ty::blob(..) => Ty::blob(len),
                    unchanged => unchanged,
                })
                .boxed()
        } else {
            prop_oneof![
                Just(Ty::u8),
                Just(Ty::u16),
                Just(Ty::u32),
                Just(Ty::u64),
                Just(Ty::u128),
                Just(Ty::i8),
                Just(Ty::i16),
                Just(Ty::i32),
                Just(Ty::i64),
                Just(Ty::i128),
                Just(Ty::f32),
                Just(Ty::f64),
                Just(Ty::bool),
            ]
            .boxed()
        }
    }
}
