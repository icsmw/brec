mod attrs;
mod extractors;
mod reserved;

use crate::*;
pub(crate) use attrs::*;
use proc_macro2::TokenStream;
pub(crate) use reserved::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub ty: Ty,
    pub injected: bool,
}

impl Field {
    pub fn injected<S: AsRef<str>>(name: S, ty: Ty) -> Self {
        Self {
            name: name.as_ref().to_string(),
            attrs: Vec::new(),
            ty,
            injected: true,
        }
    }
}

impl TryFrom<&syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
        let Some(name) = field.ident.as_ref() else {
            return Err(syn::Error::new_spanned(field, E::FailExtractIdent));
        };
        if is_reserved_field_name(name.to_string()) {
            return Err(syn::Error::new_spanned(
                name,
                E::ReservedFieldName(name.to_string()),
            ));
        }
        let mut attrs = Vec::new();
        for attr in &field.attrs {
            if let Some(attr) = Attr::try_from_attr(attr)? {
                attrs.push(attr);
            }
        }
        Ok(Self {
            name: name.to_string(),
            attrs,
            ty: Ty::try_from(&field.ty)?,
            injected: false,
        })
    }
}

impl Referred for Field {
    fn referred(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.referred();
        quote! {
            #name: #ty,
        }
    }
}

impl Static for Field {
    fn r#static(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.r#static();
        quote! {
            #name: #ty,
        }
    }
}
