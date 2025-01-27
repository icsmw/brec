use crate::*;
use proc_macro2::TokenStream;

impl StaticPacket for Ty {
    fn r#static(&self) -> TokenStream {
        let def = self.def.r#static();
        quote! {
            #def
        }
    }
}

impl StaticPacket for TyDef {
    fn r#static(&self) -> TokenStream {
        let ty = format_ident!("{}", self.to_string());
        quote! { #ty }
    }
}
