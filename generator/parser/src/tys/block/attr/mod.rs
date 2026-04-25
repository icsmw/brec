use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fmt;
use syn::Ident;

#[derive(Debug, Clone, Default)]
pub struct BlockAttrs(pub Vec<BlockAttr>);

impl BlockAttrs {
    pub fn fullpath(&self, name: Ident) -> Result<TokenStream, E> {
        let Some(BlockAttr::Path(path)) = self
            .0
            .iter()
            .find(|attr| matches!(attr, BlockAttr::Path(..)))
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
    pub fn is_no_crc(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, BlockAttr::NoCrc))
    }
}

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum BlockAttr {
    Path(ModulePath),
    NoCrc,
}

impl fmt::Display for BlockAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Path(path) => format!("{}({path})", self.id()),
                Self::NoCrc => BlockAttrId::NoCrc.to_string(),
            }
        )
    }
}
