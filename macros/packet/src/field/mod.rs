mod attrs;
mod gen;

use std::convert::TryFrom;

use crate::*;
pub(crate) use attrs::*;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub ty: Ty,
}

impl TryFrom<&syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
        let Some(name) = field.ident.as_ref() else {
            return Err(syn::Error::new_spanned(field, E::FailExtractIdent));
        };
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
        })
    }
}
