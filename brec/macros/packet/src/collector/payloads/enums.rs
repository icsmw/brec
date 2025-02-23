use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for pl in payloads.iter() {
        let fullname = pl.fullname()?;
        let fullpath = pl.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    Ok(quote! {
        pub enum Payload {
            #(#variants,)*
            Bytes(Vec<u8>),
            String(String),
        }

        impl brec::PayloadInnerDef for Payload {}

        impl brec::PayloadDef<Payload> for Payload {}
    })
}
