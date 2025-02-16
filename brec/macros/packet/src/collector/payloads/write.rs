use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn writing_to(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut write = Vec::new();
    let mut write_all = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        write.push(quote! {Payload::#fullname(pl) => pl.write(buf)});
        write_all.push(quote! {Payload::#fullname(pl) => pl.write_all(buf)});
    }
    Ok(quote! {
        impl brec::WritingPayloadTo for Payload {
            fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
                use brec::WritePayloadTo;
                match self {
                    #(#write,)*
                }
            }

            fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
                use brec::WritePayloadTo;
                match self {
                    #(#write_all,)*
                }
            }
        }
    })
}

pub fn writing_vectored_to(payloads: &[Payload]) -> Result<TokenStream, E> {
    let mut slices = Vec::new();
    for payload in payloads.iter() {
        let fullname = payload.fullname()?;
        slices.push(quote! {Payload::#fullname(pl) => pl.slices()});
    }
    Ok(quote! {
        impl brec::WritingVectoredPayloadTo for Payload {
            fn slices(&mut self) -> std::io::Result<brec::IoSlices> {
                use brec::WriteVectoredPayloadTo;
                match self {
                    #(#slices,)*
                }
            }
        }
    })
}
