mod attr;
mod field;
mod ty;

pub use attr::*;
pub use field::*;
pub use ty::*;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug, Clone)]
pub enum PayloadFields {
    Named(Vec<PayloadField>),
    Unnamed(Vec<PayloadTy>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct PayloadVariant {
    pub name: String,
    pub fields: PayloadFields,
}

#[derive(Debug, Clone)]
pub enum PayloadKind {
    Struct(PayloadFields),
    Enum(Vec<PayloadVariant>),
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub name: String,
    pub attrs: PayloadAttrs,
    pub derives: Derives,
    pub kind: PayloadKind,
}

impl Payload {
    pub fn new(name: String, attrs: PayloadAttrs, derives: Derives, kind: PayloadKind) -> Self {
        Self {
            name,
            attrs,
            derives,
            kind,
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
