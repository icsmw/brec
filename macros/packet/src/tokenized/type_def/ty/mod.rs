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
            Self::Option(ty) => {
                let inner_ty = ty.referenced();
                quote! { Option< #inner_ty > }
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
            Self::Option(ty) => {
                let inner_ty = ty.referenced();
                quote! { Option<&'a #inner_ty> }
            }
        }
    }
}
