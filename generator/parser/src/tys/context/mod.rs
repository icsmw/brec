mod attr;

pub use attr::*;

use crate::*;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::Ident;

#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    pub attrs: ContextAttrs,
    pub derives: Derives,
    pub kind: PayloadKind,
}

impl Context {
    pub fn new(name: String, attrs: ContextAttrs, derives: Derives, kind: PayloadKind) -> Self {
        Self {
            name,
            attrs,
            derives,
            kind,
        }
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
