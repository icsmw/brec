mod attr;

use syn::Visibility;

use crate::*;
use std::convert::{TryFrom, TryInto};

impl TryFrom<&mut syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: &mut syn::Field) -> Result<Self, Self::Error> {
        let Some(name) = field.ident.as_ref() else {
            return Err(syn::Error::new_spanned(field, E::FailExtractIdent));
        };
        if Field::is_reserved_name(name.to_string()) {
            return Err(syn::Error::new_spanned(
                name,
                E::ReservedFieldName(name.to_string()),
            ));
        }
        field.attrs.retain(|attr| !FieldAttr::has(attr));
        let mut attrs = Vec::new();
        for attr in &field.attrs {
            if FieldAttr::has(attr) {
                attrs.push(attr.try_into()?);
            }
        }
        Ok(Self {
            name: name.to_string(),
            attrs,
            ty: Ty::try_from(&field.ty)?,
            injected: false,
            public: matches!(field.vis, Visibility::Public(..)),
        })
    }
}
