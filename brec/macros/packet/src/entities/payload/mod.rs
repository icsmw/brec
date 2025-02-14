mod attr;

pub(crate) use attr::*;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitInt};

pub(crate) const PAYLOAD_SIG_LEN: usize = 4;
pub(crate) const PAYLOAD_CRC_LEN: usize = 4;

#[derive(Debug, Clone)]
pub struct Payload {
    pub name: String,
    pub attrs: PayloadAttrs,
}

impl Payload {
    pub fn new(name: String, attrs: PayloadAttrs) -> Self {
        Self { name, attrs }
    }
    pub fn sig(&self) -> Result<TokenStream, E> {
        let mut hasher = Hasher::new();
        hasher.update(self.fullname()?.to_string().as_bytes());
        let sig = hasher.finalize().to_le_bytes();
        Ok(quote! { [#(#sig),*] })
    }
    pub fn sig_len(&self) -> TokenStream {
        let len_lit = LitInt::new(&PAYLOAD_SIG_LEN.to_string(), proc_macro2::Span::call_site());
        quote! { #len_lit }
    }
    pub fn const_sig_name(&self) -> Ident {
        format_ident!("{}", self.name.to_ascii_uppercase())
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
