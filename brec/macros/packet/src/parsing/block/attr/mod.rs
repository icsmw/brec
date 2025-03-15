use std::convert::TryFrom;

use crate::*;
use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Token,
};

impl Parse for BlockAttrs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut attrs: Vec<BlockAttr> = vec![];
        for expr in Punctuated::<Expr, Token![,]>::parse_terminated(input)? {
            match expr {
                Expr::Assign(assign) => {
                    let Expr::Path(key) = assign.left.as_ref() else {
                        return Err(syn::Error::new_spanned(assign.left, E::FailExtractIdent));
                    };
                    let key = key
                        .path
                        .get_ident()
                        .ok_or(syn::Error::new_spanned(
                            assign.left.clone(),
                            E::FailExtractIdent,
                        ))?
                        .to_string();
                    if key == BlockAttrId::Path.to_string() {
                        attrs.push(BlockAttr::Path(ModulePath::try_from(&*assign.right)?));
                    } else {
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    }
                }
                Expr::Path(expr) => {
                    if let Some(ident) = expr.path.clone().get_ident() {
                        let as_str = ident.to_string();
                        if as_str == BlockAttrId::NoCrc.to_string() {
                            attrs.push(BlockAttr::NoCrc)
                        }
                    } else {
                        attrs.push(BlockAttr::Path(ModulePath::from(&expr)));
                    }
                }
                unknown => {
                    return Err(syn::Error::new_spanned(unknown, E::UnsupportedAttr));
                }
            }
        }
        Ok(BlockAttrs(attrs))
    }
}
