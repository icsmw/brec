use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[derive(Debug, Clone, Default)]
pub struct ContextAttrs;

impl ContextAttrs {
    pub fn fullpath(&self, name: Ident) -> Result<TokenStream, E> {
        Ok(quote! {#name})
    }

    pub fn fullname(&self, name: Ident) -> Result<Ident, E> {
        Ok(name)
    }
}
