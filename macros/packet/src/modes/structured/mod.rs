mod block;

use crate::*;
use proc_macro2::TokenStream;

pub trait Structured {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait StructuredBase {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait StructuredRead {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait StructuredReadFromSlice {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait StructuredWrite {
    fn gen(&self) -> Result<TokenStream, E>;
}
