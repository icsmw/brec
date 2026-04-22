use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(
    payloads: &[&Payload],
    derives: Vec<TokenStream>,
    cfg: &Config,
) -> Result<TokenStream, E> {
    let contexts = payloads_context(payloads)?;
    let context_def = if contexts.is_empty() {
        quote! {
            pub type PayloadContext<'a> = ();
        }
    } else {
        quote! {
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            pub enum PayloadContext<'a> {
                None,
                #(#contexts,)*
            }
        }
    };
    let payloads = payloads
        .iter()
        .copied()
        .filter(|pl| !pl.attrs.is_ctx())
        .collect::<Vec<_>>();
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
    let napi_impl = {
        #[cfg(feature = "napi")]
        {
            brec_in_node_gen::collector::payload::generate_impl(&payloads, cfg)?
        }
        #[cfg(not(feature = "napi"))]
        {
            quote! {}
        }
    };
    let wasm_impl = if cfg!(feature = "wasm") {
        integrations::collector::payloads::wasm::generate_impl(&payloads, cfg)?
    } else {
        quote! {}
    };
    let java_impl = if cfg!(feature = "java") {
        integrations::collector::payloads::java::generate_impl(&payloads, cfg)?
    } else {
        quote! {}
    };
    let csharp_impl = if cfg!(feature = "csharp") {
        integrations::collector::payloads::csharp::generate_impl(&payloads, cfg)?
    } else {
        quote! {}
    };
    Ok(quote! {
        #context_def

        #derives
        #[allow(non_snake_case)]
        pub enum Payload {
            #(#variants,)*
            #deafults
        }

        impl brec::PayloadSchema for Payload {
            type Context<'a> = PayloadContext<'a>;
        }

        impl brec::PayloadHooks for Payload {}

        impl brec::PayloadInnerDef for Payload {}

        impl brec::PayloadDef<Payload> for Payload {}
        #napi_impl
        #wasm_impl
        #java_impl
        #csharp_impl

    })
}

fn payloads_context(payloads: &[&Payload]) -> Result<Vec<TokenStream>, E> {
    let mut variants = Vec::new();
    let mut has_crypt = false;
    for payload in payloads.iter() {
        if payload.attrs.is_crypt() {
            has_crypt = true;
        }
        if payload.attrs.is_ctx() {
            let fullname = payload.fullname()?;
            let fullpath = payload.fullpath()?;
            variants.push(quote! {#fullname(&'a mut #fullpath)});
        }
    }
    if has_crypt {
        variants.push(quote! {Encrypt(&'a mut brec::prelude::EncryptOptions)});
        variants.push(quote! {Decrypt(&'a mut brec::prelude::DecryptOptions)});
    }
    Ok(variants)
}
