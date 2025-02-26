mod attr;

use crate::*;
use std::convert::TryFrom;
use syn::{Data, DeriveInput, Fields};

pub const BLOCK_ATTR: &str = "block";

impl TryFrom<(BlockAttrs, &mut DeriveInput)> for Block {
    type Error = syn::Error;
    fn try_from((attrs, input): (BlockAttrs, &mut DeriveInput)) -> Result<Self, Self::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        input.attrs.retain(|attr| !attr.path().is_ident(BLOCK_ATTR));
        let mut extracted = Vec::new();
        let Data::Struct(data_struct) = &mut input.data else {
            return Err(syn::Error::new_spanned(
                &input,
                E::NotSupportedBy(BLOCK_ATTR.to_string()),
            ));
        };
        let Fields::Named(fields) = &mut data_struct.fields else {
            return Err(syn::Error::new_spanned(
                &data_struct.fields,
                E::NotSupportedBy(BLOCK_ATTR.to_string()),
            ));
        };
        for field in &mut fields.named {
            extracted.push(Field::try_from(field)?);
        }
        extracted.insert(0, Field::injected(FIELD_SIG, Ty::blob(4)));
        extracted.push(Field::injected(FIELD_CRC, Ty::blob(4)));
        let blk = Self::new(name.to_string(), extracted, attrs, (&*input).into());
        Collector::get()
            .map_err(|err| syn::Error::new_spanned(&input, err))?
            .add_block(blk.clone())
            .map_err(|err| syn::Error::new_spanned(&input, err))?;
        Ok(blk)
    }
}
