use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn encode(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => brec::PayloadEncode::encode(pl, _ctx)});
    }
    Ok(quote! {
        impl brec::PayloadEncode for Payload {
            fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => brec::PayloadEncode::encode(pl, &mut brec::default_payload_context()),
                    Payload::String(pl) => brec::PayloadEncode::encode(pl, &mut brec::default_payload_context()),
                }
            }
        }
    })
}

pub fn encode_referred(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants
            .push(quote! {Payload::#fullname(pl) => brec::PayloadEncodeReferred::encode(pl, _ctx)});
    }
    Ok(quote! {
        impl brec::PayloadEncodeReferred for Payload {
            fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => brec::PayloadEncodeReferred::encode(pl, &mut brec::default_payload_context()),
                    Payload::String(pl) => brec::PayloadEncodeReferred::encode(pl, &mut brec::default_payload_context()),
                }
            }
        }
    })
}

pub fn sig(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => pl.sig()});
    }
    Ok(quote! {
        impl brec::PayloadSignature for Payload {
            fn sig(&self) -> brec::ByteBlock {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => pl.sig(),
                    Payload::String(pl) => pl.sig(),
                }
            }
        }
    })
}

pub fn crc(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => pl.crc(_ctx)});
    }
    Ok(quote! {
        impl brec::PayloadCrc for Payload {
            fn crc(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<brec::ByteBlock> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => pl.crc(&mut brec::default_payload_context()),
                    Payload::String(pl) => pl.crc(&mut brec::default_payload_context()),
                }
            }
        }
    })
}

pub fn size(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => pl.size(_ctx)});
    }
    Ok(quote! {
        impl brec::PayloadSize for Payload {
            fn size(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<u64> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => pl.size(&mut brec::default_payload_context()),
                    Payload::String(pl) => pl.size(&mut brec::default_payload_context()),
                }
            }
        }
    })
}
