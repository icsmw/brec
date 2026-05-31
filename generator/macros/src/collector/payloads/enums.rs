use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(
    payloads: &[&Payload],
    contexts: &[&Context],
    derives: Vec<TokenStream>,
    cfg: &Config,
) -> Result<TokenStream, E> {
    let context_variants = payloads_context(payloads, contexts)?;
    let context_def = if context_variants.is_empty() {
        quote! {
            pub type ProtocolContext<'a> = ();
        }
    } else {
        quote! {
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            pub enum ProtocolContext<'a> {
                None,
                #(#context_variants,)*
            }
        }
    };
    let payloads = payloads
        .iter()
        .copied()
        .filter(|pl| !pl.attrs.is_include())
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
    let max_payload_len = cfg
        .get_default_max_payload_len()
        .map(|len| quote! { #len })
        .unwrap_or_else(|| quote! { brec::DEFAULT_MAX_PAYLOAD_LEN });
    let max_packet_len = cfg
        .get_default_max_packet_len()
        .map(|len| quote! { #len as u64 })
        .unwrap_or_else(|| quote! { brec::DEFAULT_MAX_PACKET_LEN });
    let initial_packet_buffer_capacity = cfg
        .get_default_initial_packet_buffer_capacity()
        .map(|capacity| {
            quote! { #capacity }
        })
        .unwrap_or_else(|| quote! { brec::DEFAULT_INITIAL_PACKET_BUFFER_CAPACITY });
    let napi_impl = {
        #[cfg(feature = "napi")]
        {
            brec_node_gen::collector::payload::generate_impl(&payloads, cfg)?
        }
        #[cfg(not(feature = "napi"))]
        {
            quote! {}
        }
    };
    let wasm_impl = {
        #[cfg(feature = "wasm")]
        {
            brec_wasm_gen::collector::payload::generate_impl(&payloads, cfg)?
        }
        #[cfg(not(feature = "wasm"))]
        {
            quote! {}
        }
    };
    let java_impl = {
        #[cfg(feature = "java")]
        {
            brec_java_gen::collector::payload::generate_impl(&payloads, cfg)?
        }
        #[cfg(not(feature = "java"))]
        {
            quote! {}
        }
    };
    let csharp_impl = {
        #[cfg(feature = "csharp")]
        {
            brec_csharp_gen::collector::payload::generate_impl(&payloads, cfg)?
        }
        #[cfg(not(feature = "csharp"))]
        {
            quote! {}
        }
    };
    Ok(quote! {
        #context_def

        #derives
        #[allow(non_snake_case)]
        pub enum Payload {
            #(#variants,)*
            #deafults
        }

        impl brec::ProtocolSchema for Payload {
            type Context<'a> = ProtocolContext<'a>;

            const MAX_PAYLOAD_LEN: u32 = #max_payload_len;

            const MAX_PACKET_LEN: u64 = #max_packet_len;

            const INITIAL_PACKET_BUFFER_CAPACITY: usize = #initial_packet_buffer_capacity;
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

fn payloads_context(payloads: &[&Payload], contexts: &[&Context]) -> Result<Vec<TokenStream>, E> {
    let mut variants = Vec::new();
    let mut has_crypt = false;
    for context in contexts.iter() {
        let fullname = context.fullname()?;
        let fullpath = context.fullpath()?;
        variants.push(quote! {#fullname(&'a mut #fullpath)});
    }
    for payload in payloads.iter() {
        if payload.attrs.is_crypt() {
            has_crypt = true;
        }
    }
    if has_crypt {
        variants.push(quote! {Encrypt(&'a mut brec::prelude::EncryptOptions)});
        variants.push(quote! {Decrypt(&'a mut brec::prelude::DecryptOptions)});
    }
    Ok(variants)
}
