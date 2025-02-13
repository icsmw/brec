mod attr;

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
        let mut extracted = Vec::new();
        let Data::Struct(data_struct) = &mut input.data else {
            return Err(syn::Error::new_spanned(
                &input,
                E::NotSupportedBy(PAYLOAD_ATTR.to_string()),
            ));
        };
        let Fields::Named(fields) = &mut data_struct.fields else {
            return Err(syn::Error::new_spanned(
                &data_struct.fields,
                E::NotSupportedBy(PAYLOAD_ATTR.to_string()),
            ));
        };
        for field in &mut fields.named {
            extracted.push(
                field
                    .ident
                    .as_ref()
                    .ok_or(syn::Error::new_spanned(&field, E::FailExtractIdent))?
                    .to_string(),
            );
        }
        let payload = Self::new(name.to_string(), extracted, attrs);
        Collector::get()
            .map_err(|err| syn::Error::new_spanned(&input, err))?
            .add_payload(payload.clone())
            .map_err(|err| syn::Error::new_spanned(&input, err))?;
        Ok(payload)
    }
}
