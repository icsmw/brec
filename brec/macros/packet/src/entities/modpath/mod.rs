use std::{convert::TryFrom, fmt};

use quote::ToTokens;
use syn::{Expr, ExprPath, Lit};

use crate::*;

#[derive(Debug, Clone)]
pub struct ModulePath {
    inner: String,
}

impl TryFrom<&Expr> for ModulePath {
    type Error = syn::Error;

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        let tk_ref = expr.clone();
        let path = match expr {
            Expr::Lit(lit) => {
                let Lit::Str(path) = &lit.lit else {
                    return Err(syn::Error::new_spanned(tk_ref, E::UnsupportedAttr));
                };
                path.to_token_stream()
            }
            Expr::Path(path) => path.to_token_stream(),
            _not_supported => {
                return Err(syn::Error::new_spanned(tk_ref, E::UnsupportedAttr));
            }
        };
        Ok(Self {
            inner: path.to_string(),
        })
    }
}

impl From<&ExprPath> for ModulePath {
    fn from(expr: &ExprPath) -> Self {
        Self {
            inner: expr.path.to_token_stream().to_string(),
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
