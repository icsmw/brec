use crate::*;
use std::convert::TryFrom;
use syn::{Data, DeriveInput, Fields};

impl TryFrom<&DeriveInput> for Packet {
    type Error = syn::Error;
    fn try_from(input: &DeriveInput) -> Result<Self, Self::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        let mut extracted = Vec::new();
        if let Data::Struct(data_struct) = &input.data {
            if let Fields::Named(fields) = &data_struct.fields {
                for field in &fields.named {
                    extracted.push(Field::try_from(field)?);
                }
            }
        }
        extracted.insert(
            0,
            Field::injected(FIELD_SIG, Ty::Slice(4, Box::new(Ty::u8))),
        );
        extracted.push(Field::injected(FIELD_CRC, Ty::u32));
        extracted.push(Field::injected(FIELD_NEXT, Ty::Slice(4, Box::new(Ty::u8))));
        Ok(Self::new(name.to_string(), extracted))
    }
}
