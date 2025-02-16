use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn extract_from(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        let fullpath = payload.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::ReadPayloadFrom<#fullpath>>::read(buf, header) {
                Ok(pl) => return Ok(Payload::#fullname(pl)),
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::ExtractPayloadFrom<Payload> for Payload {
            fn read<B: std::io::Read>(
                buf: &mut B,
                header: &brec::PayloadHeader,
            ) -> Result<Payload, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn try_extract_from(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        let fullpath = payload.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::TryReadPayloadFrom<#fullpath>>::try_read(buf, header) {
                Ok(brec::ReadStatus::Success(pl)) => {
                    return Ok(brec::ReadStatus::Success(Payload::#fullname(pl)))
                }
                Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                    return Ok(brec::ReadStatus::NotEnoughData(needed))
                }
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::TryExtractPayloadFrom<Payload> for Payload {
            fn try_read<B: std::io::Read + std::io::Seek>(
                buf: &mut B,
                header: &brec::PayloadHeader,
            ) -> Result<brec::ReadStatus<Payload>, brec::Error> {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn try_extract_from_buffered(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        let fullpath = payload.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::TryReadPayloadFromBuffered<#fullpath>>::try_read(buf, header) {
                Ok(brec::ReadStatus::Success(pl)) => {
                    return Ok(brec::ReadStatus::Success(Payload::#fullname(pl)))
                }
                Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                    return Ok(brec::ReadStatus::NotEnoughData(needed))
                }
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::TryExtractPayloadFromBuffered<Payload> for Payload {
            fn try_read<B: std::io::Read>(
                buf: &mut B,
                header: &brec::PayloadHeader,
            ) -> Result<brec::ReadStatus<Payload>, brec::Error> {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}
