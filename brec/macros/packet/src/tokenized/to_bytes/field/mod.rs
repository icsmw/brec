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
            Ty::u8 => Ok(quote! { &[self.#name] }),
            Ty::u16
            | Ty::u32
            | Ty::u64
            | Ty::u128
            | Ty::i8
            | Ty::i16
            | Ty::i32
            | Ty::i64
            | Ty::i128
            | Ty::f32
            | Ty::f64 => Ok(quote! {
                &self.#name.to_le_bytes()
            }),
            Ty::bool => Ok(quote! {
                &[self.#name as u8]
            }),
            Ty::blob(..) => Ok(quote! { #by_ref self.#name }),
        }
    }
}
