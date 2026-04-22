use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn writing_to(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut write = Vec::new();
    let mut write_all = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        write.push(
            quote! {Payload::#fullname(pl) => brec::WritePayloadWithHeaderTo::write(pl, buf, _ctx)},
        );
        write_all.push(
            quote! {Payload::#fullname(pl) => brec::WritePayloadWithHeaderTo::write_all(pl, buf, _ctx)},
        );
    }
    Ok(quote! {
        impl brec::WriteMutTo for Payload {
            fn write<T: std::io::Write>(
                &mut self,
                buf: &mut T,
                _ctx: &mut <Self as brec::PayloadSchema>::Context<'_>,
            ) -> std::io::Result<usize> {
                match self {
                    #(#write,)*
                    Payload::Bytes(pl) => brec::WritePayloadWithHeaderTo::write(pl, buf, &mut brec::default_payload_context()),
                    Payload::String(pl) => brec::WritePayloadWithHeaderTo::write(pl, buf, &mut brec::default_payload_context()),
                }
            }

            fn write_all<T: std::io::Write>(
                &mut self,
                buf: &mut T,
                _ctx: &mut <Self as brec::PayloadSchema>::Context<'_>,
            ) -> std::io::Result<()> {
                match self {
                    #(#write_all,)*
                    Payload::Bytes(pl) => brec::WritePayloadWithHeaderTo::write_all(pl, buf, &mut brec::default_payload_context()),
                    Payload::String(pl) => brec::WritePayloadWithHeaderTo::write_all(pl, buf, &mut brec::default_payload_context()),
                }
            }
        }
    })
}

pub fn writing_vectored_to(payloads: &[&Payload]) -> Result<TokenStream, E> {
    let mut slices = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        slices.push(quote! {Payload::#fullname(pl) => pl.slices(_ctx)});
    }
    Ok(quote! {
        impl brec::WriteVectoredMutTo for Payload {
            fn slices(
                &mut self,
                _ctx: &mut <Self as brec::PayloadSchema>::Context<'_>,
            ) -> std::io::Result<brec::IoSlices<'_>> {
                use brec::WriteVectoredPayloadWithHeaderTo;
                match self {
                    #(#slices,)*
                    Payload::Bytes(pl) => pl.slices(&mut brec::default_payload_context()),
                    Payload::String(pl) => pl.slices(&mut brec::default_payload_context()),
                }
            }
        }
    })
}
