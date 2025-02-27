use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{token, Visibility};

use crate::*;

#[derive(Debug, Clone, Default)]
pub enum Vis {
    Public,
    #[default]
    Private,
    Restricted(String),
}

impl Vis {
    pub fn as_token(&self) -> Result<TokenStream, E> {
        Ok(match self {
            Self::Public => token::Pub::default().into_token_stream(),
            Self::Private => TokenStream::new(),
            Self::Restricted(rstr) => {
                let vis: Visibility =
                    syn::parse_str(rstr).map_err(|_| E::FailParseVisibility(rstr.to_owned()))?;
                quote! { #vis }
            }
        })
    }
}

impl From<&DeriveInput> for Vis {
    fn from(value: &DeriveInput) -> Self {
        (&value.vis).into()
    }
}

impl From<&Visibility> for Vis {
    fn from(vis: &Visibility) -> Self {
        match vis {
            Visibility::Public(..) => Vis::Public,
            Visibility::Inherited => Vis::Private,
            Visibility::Restricted(rstr) => Vis::Restricted(rstr.to_token_stream().to_string()),
        }
    }
}
