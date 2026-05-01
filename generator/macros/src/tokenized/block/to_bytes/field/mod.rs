use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::token::And;

use crate::*;

impl ToBytes for BlockField {
    fn to_bytes(&self, blob_by_ref: bool) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        let by_ref = if blob_by_ref {
            And::default().into_token_stream()
        } else {
            TokenStream::new()
        };
        match &self.ty {
            BlockTy::U8 => Ok(quote! { &[self.#name] }),
            BlockTy::U16
            | BlockTy::U32
            | BlockTy::U64
            | BlockTy::U128
            | BlockTy::I8
            | BlockTy::I16
            | BlockTy::I32
            | BlockTy::I64
            | BlockTy::I128
            | BlockTy::F32
            | BlockTy::F64 => Ok(quote! {
                &self.#name.to_le_bytes()
            }),
            BlockTy::Bool => Ok(quote! {
                &[self.#name as u8]
            }),
            BlockTy::Blob(..) => Ok(quote! { #by_ref self.#name }),
            BlockTy::LinkedToU8(_ident) => Ok(quote! { &[(&self.#name).into()] }),
        }
    }
}
