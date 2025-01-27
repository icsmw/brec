use crate::*;
use proc_macro2::TokenStream;

impl ReferredPacket for Ty {
    fn referred(&self) -> TokenStream {
        let ty = format_ident!("{}", self.to_string());
        quote! { &'a #ty }
    }
}
