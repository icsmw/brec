use proc_macro2::TokenStream;
use syn::Ident;

use crate::*;

impl Safe for Field {
    fn safe_extr(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.safe_extr(src, from, to);
        quote! {
            let #name = #ty;
        }
    }
}
