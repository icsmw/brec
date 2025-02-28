mod attr;

pub(crate) use attr::*;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitInt};

pub(crate) const BLOCK_SIG_LEN: usize = 4;
pub(crate) const BLOCK_CRC_LEN: usize = 4;

#[derive(Debug, Clone)]
pub struct Block {
    pub name: String,
    pub fields: Vec<Field>,
    pub attrs: BlockAttrs,
    pub derives: Derives,
    pub vis: Vis,
}

impl Block {
    pub fn new(
        name: String,
        fields: Vec<Field>,
        attrs: BlockAttrs,
        derives: Derives,
        vis: Vis,
    ) -> Self {
        Self {
            name,
            fields,
            attrs,
            derives,
            vis,
        }
    }
    pub fn sig(&self) -> TokenStream {
        // TODO: might be a conflict if do not consider a path
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
        let len_lit = LitInt::new(&BLOCK_SIG_LEN.to_string(), proc_macro2::Span::call_site());
        quote! { #len_lit }
    }
    pub fn size(&self) -> usize {
        self.fields
            .iter()
            .map(|f| f.size())
            .collect::<Vec<usize>>()
            .iter()
            .sum::<usize>()
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
    pub fn vis_token(&self) -> Result<TokenStream, E> {
        self.vis.as_token()
    }
}
