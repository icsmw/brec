mod block;

use proc_macro2::TokenStream;

pub trait Structured {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredBase {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredRead {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredReadFromSlice {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredWrite {
    fn gen(&self) -> TokenStream;
}
