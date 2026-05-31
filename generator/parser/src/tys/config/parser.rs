use crate::*;
use syn::{
    Expr, Token,
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
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
                        if let Expr::Lit(expr_lit) = &*assign.right
                            && let syn::Lit::Str(lit_str) = &expr_lit.lit
                        {
                            settings.push(Setting::PayloadsDerive(lit_str.value()));
                            continue;
                        }
                        return Err(syn::Error::new_spanned(assign, E::UnsupportedAttr));
                    } else if key == SettingId::DefaultMaxPayloadLen.to_string()
                        || key == SettingId::DefaultMaxPacketLen.to_string()
                    {
                        let value = if let Expr::Lit(expr_lit) = assign.right.as_ref()
                            && let syn::Lit::Int(lit_int) = &expr_lit.lit
                        {
                            lit_int
                                .base10_parse::<u32>()
                                .map_err(|e| syn::Error::new_spanned(expr_lit, e.to_string()))?
                        } else {
                            return Err(syn::Error::new_spanned(assign.right, E::UnsupportedAttr));
                        };
                        if key == SettingId::DefaultMaxPayloadLen.to_string() {
                            settings.push(Setting::DefaultMaxPayloadLen(value));
                        } else {
                            settings.push(Setting::DefaultMaxPacketLen(value));
                        }
                        continue;
                    } else if key == SettingId::DefaultInitialPacketBufferCapacity.to_string() {
                        let value = if let Expr::Lit(expr_lit) = assign.right.as_ref()
                            && let syn::Lit::Int(lit_int) = &expr_lit.lit
                        {
                            lit_int
                                .base10_parse::<usize>()
                                .map_err(|e| syn::Error::new_spanned(expr_lit, e.to_string()))?
                        } else {
                            return Err(syn::Error::new_spanned(assign.right, E::UnsupportedAttr));
                        };
                        settings.push(Setting::DefaultInitialPacketBufferCapacity(value));
                        continue;
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
                        } else if as_str == SettingId::Scheme.to_string() {
                            settings.push(Setting::Scheme);
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
