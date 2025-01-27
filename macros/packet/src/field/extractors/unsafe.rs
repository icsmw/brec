use proc_macro2::TokenStream;
use syn::Ident;

use crate::*;

impl Unsafe for Field {
    fn unsafe_extr(&self, src: &Ident, offset: usize) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.unsafe_extr(src, offset);
        quote! {
            let #name = #ty;
        }
    }
}
