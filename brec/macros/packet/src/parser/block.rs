use proc_macro2::TokenStream;
use quote::quote;

use crate::*;
use std::convert::TryFrom;

pub fn parse(attrs: BlockAttrs, mut input: DeriveInput) -> TokenStream {
    let block = match Block::try_from((attrs, &mut input)) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error(),
    };
    let reflected = match codegen::Gen::gen(&block) {
        Ok(p) => p,
        Err(err) => {
            return syn::Error::new_spanned(&input, err).to_compile_error();
        }
    };
    if let Err(err) = modificators::attrs::inject_repr_c(&mut input) {
        return syn::Error::new_spanned(&input, err).to_compile_error();
    }
    quote! {
        #input

        #reflected
    }
}
