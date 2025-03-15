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
            Ty::U8 => Ok(quote! {
               let mut #name = [0u8; 1];
               #src.read_exact(&mut #name)?;
               let #name = #name[0];
            }),
            Ty::U16
            | Ty::U32
            | Ty::U64
            | Ty::U128
            | Ty::I8
            | Ty::I16
            | Ty::I32
            | Ty::I64
            | Ty::I128
            | Ty::F32
            | Ty::F64 => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #ty::from_le_bytes(#name);
            }),
            Ty::Bool => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #name[0] != 0;
            }),
            Ty::Blob(len) => Ok(quote! {
               let mut #name = [0u8; #len];
               #src.read_exact(&mut #name)?;
            }),
            Ty::LinkedToU8(enum_name) => {
                let ident = self.ty.direct();
                Ok(quote! {
                   let mut #name = [0u8; 1];
                   #src.read_exact(&mut #name)?;
                   let #name = #ident::try_from(#name[0]).map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?;
                })
            }
        }
    }
}
