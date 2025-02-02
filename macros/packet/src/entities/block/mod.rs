mod attr;

pub(crate) use attr::*;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitInt};

#[derive(Debug, Clone)]
pub struct Block {
    pub name: String,
    pub fields: Vec<Field>,
    pub attrs: BlockAttrs,
}

impl Block {
    pub fn new(name: String, fields: Vec<Field>, attrs: BlockAttrs) -> Self {
        Self {
            name,
            fields,
            attrs,
        }
    }
    pub fn sig(&self) -> TokenStream {
        let mut hasher = Hasher::new();
        let snap = format!(
            "{};{}",
            self.name,
            self.fields
                .iter()
                .map(|f| format!("{}:{}", f.name, f.ty))
                .collect::<Vec<String>>()
                .join(";")
        );
        hasher.update(snap.as_bytes());
        let sig = hasher.finalize().to_le_bytes();
        quote! { [#(#sig),*] }
    }
    pub fn sig_len(&self) -> TokenStream {
        let len_lit = LitInt::new("4", proc_macro2::Span::call_site());
        quote! { #len_lit }
    }
    pub fn const_sig_name(&self) -> Ident {
        format_ident!("{}", self.name.to_ascii_uppercase())
    }
    pub fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }
    pub fn referred_name(&self) -> Ident {
        format_ident!("{}Referred", self.name())
    }
    pub fn fullname(&self) -> Result<Ident, E> {
        self.attrs.fullname(self.name())
    }
    pub fn fullpath(&self) -> Result<TokenStream, E> {
        self.attrs.fullpath(self.name())
    }
}
