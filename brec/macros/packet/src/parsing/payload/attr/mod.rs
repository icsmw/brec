use crate::*;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Path, Token,
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
                        let Expr::Path(path) = *assign.right else {
                            return Err(syn::Error::new_spanned(assign.right, E::UnsupportedAttr));
                        };
                        let path: Path = path.path;
                        attrs.push(PayloadAttr::Path(quote! { #path }.to_string()));
                    } else {
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    }
                }
                Expr::Path(path) => {
                    let path: Path = path.path;
                    if let Some(ident) = path.get_ident() {
                        let as_str = ident.to_string();
                        if as_str == PayloadAttrId::NoDefaultSig.to_string() {
                            attrs.push(PayloadAttr::NoDefaultSig)
                        } else if as_str == PayloadAttrId::Bincode.to_string() {
                            attrs.push(PayloadAttr::Bincode)
                        }
                    } else {
                        attrs.push(PayloadAttr::Path(quote! { #path }.to_string()));
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
