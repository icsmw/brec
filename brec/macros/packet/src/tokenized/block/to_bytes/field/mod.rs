use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::token::And;

use crate::*;

impl ToBytes for Field {
    fn to_bytes(&self, blob_by_ref: bool) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        let by_ref = if blob_by_ref {
            And::default().into_token_stream()
        } else {
            TokenStream::new()
        };
        match &self.ty {
            Ty::U8 => Ok(quote! { &[self.#name] }),
            Ty::U16
            | Ty::U32
            | Ty::U64
            | Ty::U128
            | Ty::I8
            | Ty::I16
            | Ty::I32
            | Ty::I64
            | Ty::I128
            | Ty::F32
            | Ty::F64 => Ok(quote! {
                &self.#name.to_le_bytes()
            }),
            Ty::Bool => Ok(quote! {
                &[self.#name as u8]
            }),
            Ty::Blob(..) => Ok(quote! { #by_ref self.#name }),
            Ty::LinkedToU8(_ident) => Ok(quote! { &[(&self.#name).into()] }),
        }
    }
}
