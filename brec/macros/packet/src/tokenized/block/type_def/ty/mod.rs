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
            Self::blob(len) => {
                quote! { [u8; #len] }
            }
            Self::linkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { #ident }
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
            Self::blob(len) => {
                quote! { &'a [u8; #len] }
            }
            Self::linkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { &'a #ident }
            }
        }
    }
}
