use crate::*;
use proc_macro2::TokenStream;

impl ReferredPacket for Ty {
    fn referred(&self) -> TokenStream {
        let def = self.def.referred();
        quote! {
            &'a #def
        }
    }
}

impl ReferredPacket for TyDef {
    fn referred(&self) -> TokenStream {
        let ty = format_ident!("{}", self.to_string());
        quote! { #ty }
    }
}
