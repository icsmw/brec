use proc_macro2::TokenStream;
use quote::quote;

use crate::*;
use std::convert::TryFrom;

pub fn parse(attrs: ContextAttrs, mut input: DeriveInput) -> TokenStream {
    match Context::try_from((attrs, &mut input)) {
        Ok(_) => quote! { #input },
        Err(err) => err.to_compile_error(),
    }
}
