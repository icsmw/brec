mod ty;

use proc_macro2::TokenStream;

pub trait TypeDefinition {
    fn direct(&self) -> TokenStream;
    fn referenced(&self) -> TokenStream;
}
