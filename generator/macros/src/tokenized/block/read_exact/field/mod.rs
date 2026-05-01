use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::*;

impl ReadExact for BlockField {
    fn read_exact(&self, src: &Ident) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.direct();
        let len = self.ty.size();
        match &self.ty {
            BlockTy::U8 => Ok(quote! {
               let mut #name = [0u8; 1];
               #src.read_exact(&mut #name)?;
               let #name = #name[0];
            }),
            BlockTy::U16
            | BlockTy::U32
            | BlockTy::U64
            | BlockTy::U128
            | BlockTy::I8
            | BlockTy::I16
            | BlockTy::I32
            | BlockTy::I64
            | BlockTy::I128
            | BlockTy::F32
            | BlockTy::F64 => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #ty::from_le_bytes(#name);
            }),
            BlockTy::Bool => Ok(quote! {
                let mut #name = [0u8; #len];
                #src.read_exact(&mut #name)?;
                let #name = #name[0] != 0;
            }),
            BlockTy::Blob(len) => Ok(quote! {
               let mut #name = [0u8; #len];
               #src.read_exact(&mut #name)?;
            }),
            BlockTy::LinkedToU8(enum_name) => {
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
