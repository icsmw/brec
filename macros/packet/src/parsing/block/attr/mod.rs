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
                        let Expr::Path(path) = *assign.right else {
                            return Err(syn::Error::new_spanned(assign.right, E::UnsupportedAttr));
                        };
                        let path: Path = path.path;
                        attrs.push(BlockAttr::Path(quote! { #path }.to_string()));
                    } else {
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    }
                }
                Expr::Path(path) => {
                    let path: Path = path.path;
                    attrs.push(BlockAttr::Path(quote! { #path }.to_string()));
                }
                unknown => {
                    return Err(syn::Error::new_spanned(unknown, E::UnsupportedAttr));
                }
            }
        }
        Ok(BlockAttrs(attrs))
    }
}
