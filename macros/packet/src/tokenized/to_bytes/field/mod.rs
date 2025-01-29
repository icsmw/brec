use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::*;

impl ToBytes for Field {
    fn to_bytes(&self) -> Result<TokenStream, E> {
        let name = format_ident!("{}", self.name);
        match &self.ty {
            Ty::u8 | Ty::i8 => Ok(quote! { &[self.#name] }),
            Ty::u16
            | Ty::u32
            | Ty::u64
            | Ty::u128
            | Ty::i16
            | Ty::i32
            | Ty::i64
            | Ty::i128
            | Ty::f32
            | Ty::f64 => Ok(quote! {
                self.#name.to_le_bytes()
            }),
            Ty::bool => Ok(quote! {
                &[self.#name as u8]
            }),
            Ty::Slice(len, ty) => match **ty {
                Ty::u8 | Ty::i8 => Ok(quote! { self.#name }),
                Ty::u16
                | Ty::u32
                | Ty::u64
                | Ty::u128
                | Ty::i16
                | Ty::i32
                | Ty::i64
                | Ty::i128
                | Ty::f32
                | Ty::f64 => {
                    let size = ty.size();
                    let len = len * size;
                    Ok(quote! { {
                        let bytes = [u8; #len];
                        for (n, &p) in self.#name.iter().enumerate() {
                            buffer[n * #size..n * #size + #size].copy_from_slice(&p.to_le_bytes());
                        }
                        bytes
                    } })
                }
                Ty::bool => Ok(quote! { {
                    let bytes = [u8; #len];
                    for (n, &p) in self.#name.iter().enumerate() {
                        buffer[n] = p as u8;
                    }
                    bytes
                } }),
                Ty::Slice(..) | Ty::Option(..) => Err(E::UnsupportedTypeInSlice),
            },
            Ty::Option(ty) => {
                todo!("Add wrapper")
            }
        }
    }
}
