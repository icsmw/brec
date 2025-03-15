use std::convert::TryFrom;

use crate::*;
use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Token,
};

impl Parse for PayloadAttrs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut attrs: Vec<PayloadAttr> = vec![];
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
                    if key == PayloadAttrId::Path.to_string() {
                        attrs.push(PayloadAttr::Path(ModulePath::try_from(&*assign.right)?));
                    } else {
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    }
                }
                Expr::Path(expr) => {
                    if let Some(ident) = expr.path.clone().get_ident() {
                        let as_str = ident.to_string();
                        if as_str == PayloadAttrId::NoDefaultSig.to_string() {
                            attrs.push(PayloadAttr::NoDefaultSig)
                        } else if as_str == PayloadAttrId::Bincode.to_string() {
                            attrs.push(PayloadAttr::Bincode)
                        } else if as_str == PayloadAttrId::Hooks.to_string() {
                            attrs.push(PayloadAttr::Hooks)
                        } else if as_str == PayloadAttrId::NoAutoCrc.to_string() {
                            attrs.push(PayloadAttr::NoAutoCrc)
                        } else if as_str == PayloadAttrId::NoCrc.to_string() {
                            attrs.push(PayloadAttr::NoCrc)
                        }
                    } else {
                        attrs.push(PayloadAttr::Path(ModulePath::from(&expr)));
                    }
                }
                unknown => {
                    return Err(syn::Error::new_spanned(unknown, E::UnsupportedAttr));
                }
            }
        }
        Ok(PayloadAttrs(attrs))
    }
}
