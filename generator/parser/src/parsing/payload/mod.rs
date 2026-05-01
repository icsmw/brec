mod attr;
mod ty;

use crate::*;
use std::convert::TryFrom;
use syn::{Data, DeriveInput, Fields};

pub const PAYLOAD_ATTR: &str = "payload";

impl TryFrom<(PayloadAttrs, &mut DeriveInput)> for Payload {
    type Error = syn::Error;
    fn try_from((attrs, input): (PayloadAttrs, &mut DeriveInput)) -> Result<Self, Self::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        input
            .attrs
            .retain(|attr| !attr.path().is_ident(PAYLOAD_ATTR));
        let kind = PayloadKind::try_from(&input.data)?;
        let payload = Self::new(name.to_string(), attrs, (&*input).into(), kind);
        Collector::get()
            .map_err(|err| syn::Error::new_spanned(&input, err))?
            .add_payload(payload.clone())
            .map_err(|err| syn::Error::new_spanned(&input, err))?;
        Ok(payload)
    }
}

impl TryFrom<&Data> for PayloadKind {
    type Error = syn::Error;

    fn try_from(data: &Data) -> Result<Self, Self::Error> {
        match data {
            Data::Struct(data_struct) => Ok(PayloadKind::Struct(PayloadFields::try_from(
                &data_struct.fields,
            )?)),
            Data::Enum(data_enum) => Ok(PayloadKind::Enum(
                data_enum
                    .variants
                    .iter()
                    .map(|variant| {
                        Ok(PayloadVariant {
                            name: variant.ident.to_string(),
                            fields: PayloadFields::try_from(&variant.fields)?,
                        })
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?,
            )),
            _ => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                E::NotSupportedBy(PAYLOAD_ATTR.to_string()),
            )),
        }
    }
}

impl TryFrom<&Fields> for PayloadFields {
    type Error = syn::Error;

    fn try_from(fields: &Fields) -> Result<Self, Self::Error> {
        match fields {
            Fields::Named(fields) => Ok(PayloadFields::Named(
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let Some(name) = field.ident.as_ref() else {
                            return Err(syn::Error::new_spanned(field, E::FailExtractIdent));
                        };
                        if BlockField::is_reserved_name(name.to_string()) {
                            return Err(syn::Error::new_spanned(
                                name,
                                E::ReservedFieldName(name.to_string()),
                            ));
                        }
                        Ok(PayloadField {
                            name: name.to_string(),
                            ty: PayloadTy::try_from(&field.ty)?,
                            vis: Vis::from(&field.vis),
                        })
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?,
            )),
            Fields::Unnamed(fields) => Ok(PayloadFields::Unnamed(
                fields
                    .unnamed
                    .iter()
                    .map(|field| PayloadTy::try_from(&field.ty))
                    .collect::<Result<Vec<_>, syn::Error>>()?,
            )),
            Fields::Unit => Ok(PayloadFields::Unit),
        }
    }
}
