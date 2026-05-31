use crate::*;
use syn::parse::{self, Parse, ParseStream};

impl Parse for ContextAttrs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        if input.is_empty() {
            Ok(Self)
        } else {
            Err(input.error(E::UnsupportedAttr))
        }
    }
}
