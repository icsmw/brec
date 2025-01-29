use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Block {
    pub fn new(name: String, fields: Vec<Field>) -> Self {
        Self { name, fields }
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
    pub fn const_sig_name(&self) -> Ident {
        format_ident!("{}", self.name.to_ascii_uppercase())
    }
    pub fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }
    pub fn referred_name(&self) -> Ident {
        format_ident!("{}Referred", self.name())
    }
}
