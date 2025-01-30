use crate::*;
use std::convert::TryFrom;
use syn::{Data, DeriveInput, Fields};

pub const BLOCK_ATTR: &str = "block";

impl TryFrom<&mut DeriveInput> for Block {
    type Error = syn::Error;
    fn try_from(input: &mut DeriveInput) -> Result<Self, Self::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        input.attrs.retain(|attr| !attr.path().is_ident(BLOCK_ATTR));
        let mut extracted = Vec::new();
        if let Data::Struct(data_struct) = &mut input.data {
            if let Fields::Named(fields) = &mut data_struct.fields {
                for field in &mut fields.named {
                    extracted.push(Field::try_from(field)?);
                }
            }
        }
        extracted.insert(
            0,
            Field::injected(FIELD_SIG, Ty::Slice(4, Box::new(Ty::u8))),
        );
        extracted.push(Field::injected(FIELD_CRC, Ty::u32));
        Ok(Self::new(name.to_string(), extracted))
    }
}
