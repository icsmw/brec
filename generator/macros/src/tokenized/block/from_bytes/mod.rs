mod field;
mod ty;

use proc_macro2::TokenStream;
use syn::Ident;

pub trait FromBytes {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream;
}
