use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::*;

impl FromBytes for BlockTy {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        match self {
            BlockTy::U8
            | BlockTy::U16
            | BlockTy::U32
            | BlockTy::U64
            | BlockTy::U128
            | BlockTy::I8
            | BlockTy::I16
            | BlockTy::I32
            | BlockTy::I64
            | BlockTy::I128
            | BlockTy::F32
            | BlockTy::F64 => {
                let ty = self.direct();
                quote! {
                   #ty::from_le_bytes(#src[#from..#to].try_into()?)
                }
            }
            BlockTy::Bool => {
                quote! {
                   u8::from_le_bytes(#src[#from..#to].try_into()?) == 1
                }
            }
            BlockTy::Blob(len) => quote! {
                <&[u8; #len]>::try_from(&#src[#from..#to])?
            },
            BlockTy::LinkedToU8(enum_name) => {
                let ident = self.direct();
                quote! {
                    #ident::try_from(u8::from_le_bytes(#src[#from..#to].try_into()?))
                        .map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?
                }
            }
        }
    }
}
