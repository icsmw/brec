use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::*;

impl ToBytes for Field {
    fn to_bytes(&self) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        match &self.ty {
            Ty::u8 => Ok(quote! { &[self.#name] }),
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
                &self.#name.to_le_bytes()
            }),
            Ty::bool => Ok(quote! {
                &[self.#name as u8]
            }),
            Ty::Slice(len, ty) => match **ty {
                Ty::u8 => Ok(quote! { &self.#name }),
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
                | Ty::f64 => {
                    let size = ty.size();
                    let len = len * size;
                    Ok(quote! { {
                        let mut bytes = [0u8; #len];
                        for (n, &p) in self.#name.iter().enumerate() {
                            bytes[n * #size..n * #size + #size].copy_from_slice(&p.to_le_bytes());
                        }
                        bytes
                    } })
                }
                Ty::bool => Ok(quote! { {
                    let mut bytes = [0u8; #len];
                    for (n, &p) in self.#name.iter().enumerate() {
                        bytes[n] = p as u8;
                    }
                    bytes
                } }),
                Ty::Slice(..) => Err(E::UnsupportedTypeInSlice),
            },
        }
    }
}
