mod field;

use proc_macro2::TokenStream;

pub trait Assignation {
    fn direct_ty(&self) -> TokenStream;
    fn referenced_ty(&self) -> TokenStream;
}
