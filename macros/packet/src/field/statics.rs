use crate::*;
use proc_macro2::TokenStream;

impl StaticPacket for Field {
    fn r#static(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.r#static();
        quote! {
            #name: #ty,
        }
    }
}
