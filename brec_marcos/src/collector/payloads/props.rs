use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn encode(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => brec::PayloadEncode::encode(pl)});
    }
    Ok(quote! {
        impl brec::PayloadEncode for Payload {
            fn encode(&self) -> std::io::Result<Vec<u8>> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => brec::PayloadEncode::encode(pl),
                    Payload::String(pl) => brec::PayloadEncode::encode(pl),
                }
            }
        }
    })
}

pub fn encode_referred(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => brec::PayloadEncodeReferred::encode(pl)});
    }
    Ok(quote! {
        impl brec::PayloadEncodeReferred for Payload {
            fn encode(&self) -> std::io::Result<Option<&[u8]>> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => brec::PayloadEncodeReferred::encode(pl),
                    Payload::String(pl) => brec::PayloadEncodeReferred::encode(pl),
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
        variants.push(quote! {Payload::#fullname(pl) => pl.crc()});
    }
    Ok(quote! {
        impl brec::PayloadCrc for Payload {
            fn crc(&self) -> std::io::Result<brec::ByteBlock> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => pl.crc(),
                    Payload::String(pl) => pl.crc(),
                }
            }
        }
    })
}

pub fn size(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        variants.push(quote! {Payload::#fullname(pl) => pl.size()});
    }
    Ok(quote! {
        impl brec::PayloadSize for Payload {
            fn size(&self) -> std::io::Result<u64> {
                match self {
                    #(#variants,)*
                    Payload::Bytes(pl) => pl.size(),
                    Payload::String(pl) => pl.size(),
                }
            }
        }
    })
}
