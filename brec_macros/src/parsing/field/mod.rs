use crate::*;
use std::convert::TryFrom;

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
        Ok(Self {
            name: name.to_string(),
            ty: Ty::try_from(&field.ty)?,
            injected: false,
            vis: Vis::from(&field.vis),
        })
    }
}
