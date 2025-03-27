use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(
    payloads: &[&Payload],
    derives: Vec<TokenStream>,
    cfg: &Config,
) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for pl in payloads.iter() {
        let fullname = pl.fullname()?;
        let fullpath = pl.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    let derives = [derives, cfg.get_payload_derive()?].concat();
    let derives = if derives.is_empty() {
        quote! {}
    } else {
        quote! {#[derive(#(#derives,)*)]}
    };
    let deafults = if cfg.is_no_default_payloads() {
        quote! {}
    } else {
        quote! {
            Bytes(Vec<u8>),
            String(String),
        }
    };
    Ok(quote! {
        #derives
        #[allow(non_snake_case)]
        pub enum Payload {
            #(#variants,)*
            #deafults
        }

        impl brec::PayloadHooks for Payload {}

        impl brec::PayloadInnerDef for Payload {}

        impl brec::PayloadDef<Payload> for Payload {}

    })
}
