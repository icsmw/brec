use crate::*;
use proc_macro2::TokenStream;

impl ReferredPacket for Field {
    fn referred(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.referred();
        quote! {
            #name: #ty,
        }
    }
}
