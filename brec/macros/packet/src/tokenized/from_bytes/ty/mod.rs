use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::*;

impl FromBytes for Ty {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        match self {
            Ty::u8
            | Ty::u16
            | Ty::u32
            | Ty::u64
            | Ty::u128
            | Ty::i8
            | Ty::i16
            | Ty::i32
            | Ty::i64
            | Ty::i128
            | Ty::f32
            | Ty::f64 => {
                let ty = self.direct();
                quote! {
                   #ty::from_le_bytes(#src[#from..#to].try_into()?)
                }
            }
            Ty::bool => {
                quote! {
                   u8::from_le_bytes(#src[#from..#to].try_into()?) == 1
                }
            }
            Ty::blob(len) => quote! {
                <&[u8; #len]>::try_from(&#src[#from..#to])?
            },
        }
    }

    fn r#unsafe(&self, src: &Ident, offset: usize) -> TokenStream {
        let ty = self.direct();
        if offset == 0 {
            if matches!(self, Ty::u8) {
                quote! {
                    unsafe { &*#src.as_ptr() }
                }
            } else {
                quote! {
                    unsafe { &*(#src.as_ptr() as *const #ty) }
                }
            }
        } else if matches!(self, Ty::u8) {
            quote! {
                unsafe { &*#src.as_ptr().add(#offset) }
            }
        } else {
            quote! {
               unsafe { &*(#src.as_ptr().add(#offset) as *const #ty) }
            }
        }
    }
}
