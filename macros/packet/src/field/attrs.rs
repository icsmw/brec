use crate::*;
use std::fmt;
use syn::Attribute;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug)]
pub enum Attr {
    LinkWith(String),
}

impl Attr {
    pub fn try_from_attr(attr: &Attribute) -> Result<Option<Self>, syn::Error> {
        if attr.path().is_ident(&AttrId::LinkWith.to_string()) {
            let mut inner = None;
            attr.parse_nested_meta(|meta| {
                if let Some(ident) = meta.path.get_ident() {
                    let ident = ident.to_string();
                    if ident.is_empty() {
                        Err(syn::Error::new_spanned(attr, E::LinkingRequiresEnumName))
                    } else {
                        inner = Some(Attr::LinkWith(ident));
                        Ok(())
                    }
                } else {
                    Err(syn::Error::new_spanned(attr, E::LinkingRequiresEnumName))
                }
            })?;
            Ok(Some(inner.ok_or(syn::Error::new_spanned(
                attr,
                E::LinkingRequiresEnumName,
            ))?))
        } else {
            Ok(None)
        }
    }
}

impl fmt::Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LinkWith(target) => format!("{}({target})", self.id()),
            }
        )
    }
}
