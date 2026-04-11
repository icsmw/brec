mod attr;

pub(crate) use attr::*;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug, Clone)]
pub struct Payload {
    pub name: String,
    pub attrs: PayloadAttrs,
    pub derives: Derives,
}

impl Payload {
    pub fn new(name: String, attrs: PayloadAttrs, derives: Derives) -> Self {
        Self {
            name,
            attrs,
            derives,
        }
    }
    pub fn sig(&self) -> Result<TokenStream, E> {
        let mut hasher = Hasher::new();
        hasher.update(self.fullname()?.to_string().as_bytes());
        let sig = hasher.finalize().to_le_bytes();
        Ok(quote! { [#(#sig),*] })
    }
    pub fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }
    pub fn fullname(&self) -> Result<Ident, E> {
        self.attrs.fullname(self.name())
    }
    pub fn fullpath(&self) -> Result<TokenStream, E> {
        self.attrs.fullpath(self.name())
    }
}
