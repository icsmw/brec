use proc_macro2::TokenStream;
use syn::Ident;

mod safe;
mod r#unsafe;

pub trait Safe {
    fn safe_extr(&self, src: &Ident, from: usize, to: usize) -> TokenStream;
}

pub trait Unsafe {
    fn unsafe_extr(&self, src: &Ident, offset: usize) -> TokenStream;
}
