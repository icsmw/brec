use proc_macro2::TokenStream;
use syn::Ident;

pub trait Structured {
    fn structured(&self) -> TokenStream;
}

pub trait Reflected {
    fn reflected(&self) -> TokenStream;
}
