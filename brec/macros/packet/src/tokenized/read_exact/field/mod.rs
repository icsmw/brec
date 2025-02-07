use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::*;

impl ReadExact for Field {
    fn read_exact(&self, src: &Ident) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.direct();
        let len = self.ty.size();
        match &self.ty {
            Ty::u8 => Ok(quote! {
               let mut #name = [0u8; 1];
               #src.read_exact(&mut #name)?;
               let #name = #name[0];
            }),
            Ty::u16
            | Ty::u32
            | Ty::u64
            | Ty::u128
            | Ty::i8
            | Ty::i16
            | Ty::i32
            | Ty::i64
            | Ty::i128
            | Ty::f32
            | Ty::f64 => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #ty::from_le_bytes(#name);
            }),
            Ty::bool => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #name[0] != 0;
            }),
            Ty::blob(len) => Ok(quote! {
               let mut #name = [0u8; #len];
               #src.read_exact(&mut #name)?;
            }),
            Ty::linkedToU8(enum_name) => {
                let ident = self.ty.direct();
                Ok(quote! {
                   let mut #name = [0u8; 1];
                   #src.read_exact(&mut #name)?;
                   let level = #ident::try_from(#name[0]).map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?;
                })
            }
        }
    }
}
