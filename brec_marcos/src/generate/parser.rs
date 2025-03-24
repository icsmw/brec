use crate::*;
use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Token,
};

impl Parse for Config {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut settings: Vec<Setting> = vec![];
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
                    if key == SettingId::PayloadsDerive.to_string() {
                        if let Expr::Lit(expr_lit) = &*assign.right {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                settings.push(Setting::PayloadsDerive(lit_str.value()));
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    } else {
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    }
                }
                Expr::Path(expr) => {
                    if let Some(ident) = expr.path.clone().get_ident() {
                        let as_str = ident.to_string();
                        if as_str == SettingId::NoDefaultPayload.to_string() {
                            settings.push(Setting::NoDefaultPayload);
                            continue;
                        }
                    }
                    return Err(syn::Error::new_spanned(expr, E::UnsupportedAttr));
                }
                unknown => {
                    return Err(syn::Error::new_spanned(unknown, E::UnsupportedAttr));
                }
            }
        }
        Ok(Config(settings))
    }
}
