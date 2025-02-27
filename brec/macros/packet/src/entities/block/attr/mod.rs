use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::fmt;
use syn::{parse, parse_str, Attribute, Expr, ExprPath, Ident, Lit, Path};

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
        let fullpath = format!("{path}::{name}");
        let fullpath: Path = parse_str(&fullpath).map_err(|_err| E::FailParseFullpath)?;
        Ok(quote! { #fullpath })
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
}
#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum BlockAttr {
    Path(ModulePath),
}

impl BlockAttr {
    pub fn has(attr: &Attribute) -> bool {
        attr.path().is_ident(&BlockAttrId::Path.to_string())
    }
}

impl fmt::Display for BlockAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Path(path) => format!("{}({path})", self.id()),
            }
        )
    }
}
