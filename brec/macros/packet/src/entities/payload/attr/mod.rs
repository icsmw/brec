use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fmt;
use syn::{Attribute, Ident};

#[derive(Debug, Clone, Default)]
pub struct PayloadAttrs(pub Vec<PayloadAttr>);

impl PayloadAttrs {
    pub fn fullpath(&self, name: Ident) -> Result<TokenStream, E> {
        let Some(PayloadAttr::Path(path)) = self
            .0
            .iter()
            .find(|attr| matches!(attr, PayloadAttr::Path(..)))
        else {
            return Ok(quote! {#name});
        };
        path.join(format_ident!("{name}"))
    }
    pub fn fullname(&self, name: Ident) -> Result<Ident, E> {
        Ok(format_ident!(
            "{}",
            self.fullpath(name)?
                .to_string()
                .split("::")
                .map(|s| {
                    let mut chars = s.trim().chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<String>>()
                .join("")
        ))
    }
    pub fn no_default_sig(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, PayloadAttr::NoDefaultSig))
    }
    pub fn hooks(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, PayloadAttr::Hooks))
    }
    pub fn is_no_auto_crc(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, PayloadAttr::NoAutoCrc))
    }
    pub fn is_no_crc(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, PayloadAttr::NoCrc))
    }
    pub fn is_bincode(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, PayloadAttr::Bincode))
    }
}
#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum PayloadAttr {
    Path(ModulePath),
    NoDefaultSig,
    Hooks,
    NoAutoCrc,
    NoCrc,
    Bincode,
}

impl fmt::Display for PayloadAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Path(path) => format!("{}({path})", self.id()),
                Self::NoDefaultSig => PayloadAttrId::NoDefaultSig.to_string(),
                Self::Hooks => PayloadAttrId::Hooks.to_string(),
                Self::NoAutoCrc => PayloadAttrId::NoAutoCrc.to_string(),
                Self::NoCrc => PayloadAttrId::NoCrc.to_string(),
                Self::Bincode => PayloadAttrId::Bincode.to_string(),
            }
        )
    }
}
