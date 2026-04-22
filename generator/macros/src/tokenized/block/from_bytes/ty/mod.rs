use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::*;

impl FromBytes for Ty {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        match self {
            Ty::U8
            | Ty::U16
            | Ty::U32
            | Ty::U64
            | Ty::U128
            | Ty::I8
            | Ty::I16
            | Ty::I32
            | Ty::I64
            | Ty::I128
            | Ty::F32
            | Ty::F64 => {
                let ty = self.direct();
                quote! {
                   #ty::from_le_bytes(#src[#from..#to].try_into()?)
                }
            }
            Ty::Bool => {
                quote! {
                   u8::from_le_bytes(#src[#from..#to].try_into()?) == 1
                }
            }
            Ty::Blob(len) => quote! {
                <&[u8; #len]>::try_from(&#src[#from..#to])?
            },
            Ty::LinkedToU8(enum_name) => {
                let ident = self.direct();
                quote! {
                    #ident::try_from(u8::from_le_bytes(#src[#from..#to].try_into()?))
                        .map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?
                }
            }
        }
    }
}
