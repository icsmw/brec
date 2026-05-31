mod attr;

use crate::*;
use std::convert::TryFrom;
use syn::DeriveInput;

pub const CONTEXT_ATTR: &str = "context";

impl TryFrom<(ContextAttrs, &mut DeriveInput)> for Context {
    type Error = syn::Error;

    fn try_from((attrs, input): (ContextAttrs, &mut DeriveInput)) -> Result<Self, Self::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        input
            .attrs
            .retain(|attr| !attr.path().is_ident(CONTEXT_ATTR));
        let kind = PayloadKind::try_from(&input.data)?;
        let context = Self::new(name.to_string(), attrs, (&*input).into(), kind);
        Collector::get()
            .map_err(|err| syn::Error::new_spanned(&input, err))?
            .add_context(context.clone())
            .map_err(|err| syn::Error::new_spanned(&input, err))?;
        Ok(context)
    }
}
