use crate::*;
use std::convert::TryFrom;
use syn::Attribute;

impl TryFrom<&Attribute> for FieldAttr {
    type Error = syn::Error;
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        if attr.path().is_ident(&FieldAttrId::LinkWith.to_string()) {
            let mut inner = None;
            attr.parse_nested_meta(|meta| {
                if let Some(ident) = meta.path.get_ident() {
                    let ident = ident.to_string();
                    if ident.is_empty() {
                        Err(syn::Error::new_spanned(attr, E::LinkingRequiresEnumName))
                    } else {
                        inner = Some(FieldAttr::LinkWith(ident));
                        Ok(())
                    }
                } else {
                    Err(syn::Error::new_spanned(attr, E::LinkingRequiresEnumName))
                }
            })?;
            inner.ok_or(syn::Error::new_spanned(attr, E::LinkingRequiresEnumName))
        } else {
            Err(syn::Error::new_spanned(attr, E::NoSuitableAttr))
        }
    }
}
