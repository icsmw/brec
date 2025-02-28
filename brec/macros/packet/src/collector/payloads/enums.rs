use crate::*;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen(payloads: &[&Payload], derives: Vec<String>) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for pl in payloads.iter() {
        let fullname = pl.fullname()?;
        let fullpath = pl.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    let derives = if derives.is_empty() {
        quote! {}
    } else {
        let ders = derives.into_iter().map(|der| format_ident!("{der}"));
        quote! {#[derive(#(#ders,)*)]}
    };
    Ok(quote! {
        #derives
        pub enum Payload {
            #(#variants,)*
            Bytes(Vec<u8>),
            String(String),
        }

        impl brec::PayloadInnerDef for Payload {}

        impl brec::PayloadDef<Payload> for Payload {}
    })
}
