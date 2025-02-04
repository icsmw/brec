use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl TypeDefinition for Ty {
    fn direct(&self) -> TokenStream {
        match self {
            Self::u8
            | Self::u16
            | Self::u32
            | Self::u64
            | Self::u128
            | Self::i8
            | Self::i16
            | Self::i32
            | Self::i64
            | Self::i128
            | Self::f32
            | Self::f64
            | Self::bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { #ty }
            }
            Self::Slice(len, ty) => {
                let inner_ty = ty.direct();
                quote! { [#inner_ty; #len] }
            }
        }
    }
    fn referenced(&self) -> TokenStream {
        match self {
            Self::u8
            | Self::u16
            | Self::u32
            | Self::u64
            | Self::u128
            | Self::i8
            | Self::i16
            | Self::i32
            | Self::i64
            | Self::i128
            | Self::f32
            | Self::f64
            | Self::bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { &'a  #ty }
            }
            Self::Slice(len, ty) => {
                let inner_ty = ty.direct();
                quote! { &'a [#inner_ty; #len] }
            }
        }
    }
    fn default(&self) -> TokenStream {
        match self {
            Self::u8 => quote! { 0u8 },
            Self::u16 => quote! { 0u16 },
            Self::u32 => quote! { 0u32 },
            Self::u64 => quote! { 0u64 },
            Self::u128 => quote! { 0u128 },
            Self::i8 => quote! { 0i8 },
            Self::i16 => quote! { 0i16 },
            Self::i32 => quote! { 0i32 },
            Self::i64 => quote! { 0i64 },
            Self::i128 => quote! { 0i128 },
            Self::f32 => quote! { 0f32 },
            Self::f64 => quote! { 0f64 },
            Self::bool => quote! { false },
            Self::Slice(len, ty) => {
                let ty = ty.default();
                quote! { [#ty; #len] }
            }
        }
    }
}
