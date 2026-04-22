use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Assignation for Field {
    fn referenced_ty(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.referenced();
        quote! {
            #name: #ty,
        }
    }
    fn direct_ty(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.direct();
        quote! {
            #name: #ty,
        }
    }
}
