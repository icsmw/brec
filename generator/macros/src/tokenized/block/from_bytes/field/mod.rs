use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::*;

impl FromBytes for Field {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.safe(src, from, to);
        quote! {
            let #name = #ty;
        }
    }
}
