use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl TypeDefinition for Ty {
    fn direct(&self) -> TokenStream {
        match self {
            Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::F32
            | Self::F64
            | Self::Bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { #ty }
            }
            Self::Blob(len) => {
                quote! { [u8; #len] }
            }
            Self::LinkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { #ident }
            }
        }
    }
    fn referenced(&self) -> TokenStream {
        match self {
            Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::F32
            | Self::F64
            | Self::Bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { &'a  #ty }
            }
            Self::Blob(len) => {
                quote! { &'a [u8; #len] }
            }
            Self::LinkedToU8(ident) => {
                let ident = format_ident!("{ident}");
                quote! { &'a #ident }
            }
        }
    }
}
