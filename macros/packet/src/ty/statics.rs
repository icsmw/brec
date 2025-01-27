use crate::*;
use proc_macro2::TokenStream;

impl StaticPacket for Ty {
    fn r#static(&self) -> TokenStream {
        let ty = format_ident!("{}", self.to_string());
        quote! { #ty }
    }
}
