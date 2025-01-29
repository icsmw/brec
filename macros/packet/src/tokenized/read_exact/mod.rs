mod field;

use crate::*;
use proc_macro2::TokenStream;
use syn::Ident;

pub trait ReadExact {
    fn read_exact(&self, src: &Ident) -> Result<TokenStream, E>;
}
