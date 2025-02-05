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
            Ty::Slice(len, ty) => match **ty {
                Ty::u8 => Ok(quote! {
                   let mut #name = [0u8; #len];
                   #src.read_exact(&mut #name)?;
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
                | Ty::f64 => {
                    let size = ty.size();
                    let slen = len * size;
                    let default = ty.default();
                    let ty = ty.direct();
                    let slice = format_ident!("{}_slice", self.name);
                    Ok(quote! {
                        let mut #slice = [0u8; #slen];
                        #src.read_exact(&mut #slice)?;
                        let mut #name = [#default; #len];
                        for (i, chunk) in #slice.chunks_exact(#size).enumerate() {
                            #name[i] = #ty::from_le_bytes(chunk.try_into()?);
                        }
                    })
                }
                Ty::bool => {
                    let slice = format_ident!("{}_slice", self.name);
                    Ok(quote! {
                        let mut #slice = [0u8; #len];
                        #src.read_exact(&mut #slice)?;
                        let mut #name = [false; #len];
                        for i in 0..#len {
                            #name[i] = #slice[i] != 0;
                        }
                    })
                }
                Ty::Slice(..) => Err(E::UnsupportedTypeInSlice),
            },
        }
    }
}
