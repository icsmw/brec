use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path};

#[derive(Debug, Clone)]
pub struct Derives {
    derives: Vec<String>,
}

impl Derives {
    pub fn common(derives: Vec<&Self>) -> Result<Vec<TokenStream>, E> {
        let mut common = derives
            .first()
            .map(|der| der.derives.clone())
            .unwrap_or_default();
        derives.iter().for_each(|v| {
            common.retain(|der| v.derives.contains(der));
        });
        common
            .into_iter()
            .map(|derive| {
                let derive = derive.trim();
                syn::parse_str::<Path>(derive)
                    .map(|path| quote!(#path))
                    .map_err(|_e| E::FailParseDerive(derive.to_owned()))
            })
            .collect()
    }
}

impl From<&DeriveInput> for Derives {
    fn from(input: &DeriveInput) -> Self {
        let mut derives = Vec::new();
        input.attrs.iter().for_each(|attr| {
            if attr.path().is_ident("derive") {
                let _ = attr.parse_nested_meta(|meta| {
                    let path: &Path = &meta.path;
                    derives.push(
                        path.segments
                            .iter()
                            .map(|seg| seg.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::"),
                    );
                    Ok(())
                });
            }
        });
        Self { derives }
    }
}
