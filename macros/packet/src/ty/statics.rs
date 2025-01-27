use crate::*;
use proc_macro2::TokenStream;

impl StaticPacket for Ty {
    fn r#static(&self) -> TokenStream {
        match self {
            Self::u8
            | Self::u16
            | Self::u32
            | Self::u64
            | Self::u128
            | Self::i8
            | Self::i16
            | Self::i32
            | Self::i64
            | Self::i128
            | Self::f32
            | Self::f64
            | Self::bool => {
                let ty = format_ident!("{}", self.to_string());
                quote! { #ty }
            }
            Self::Slice(len, ty) => {
                let inner_ty = ty.r#static();
                quote! { [#inner_ty; #len] }
            }
            Self::Option(ty) => {
                let inner_ty = ty.referred();
                quote! { Option< #inner_ty > }
            }
        }
    }
}
