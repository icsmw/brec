mod packet;

use proc_macro2::TokenStream;

pub trait Structured {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredBase {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredDeserializableImpl {
    fn gen(&self) -> TokenStream;
}

pub trait StructuredSerializableImpl {
    fn gen(&self) -> TokenStream;
}
