use proc_macro2::TokenStream;
use syn::Ident;

use crate::*;

impl Safe for Ty {
    fn safe_extr(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
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
            | Ty::f64
            | Ty::bool => {
                let ty = self.r#static();
                quote! {
                   #ty::from_le_bytes(#src[#from..#to].try_into()?)
                }
            }
            Ty::Slice(len, ty) => {
                let inner_ty = ty.r#static();
                quote! {
                    <&[#inner_ty; #len]>::try_from(&#src[#from..#to])?
                }
            }
            Ty::Option(ty) => {
                let inner_ty = ty.safe_extr(src, from, to);
                quote! {
                    Some( #inner_ty )
                }
            }
        }
    }
}
