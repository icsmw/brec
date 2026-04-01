use proc_macro2::TokenStream;
use quote::quote;

use crate::*;
use std::convert::TryFrom;

pub fn parse(attrs: PayloadAttrs, mut input: DeriveInput) -> TokenStream {
    #[cfg(feature = "napi")]
    let payload_data = input.data.clone();
    #[cfg(feature = "napi")]
    let payload_name = input.ident.clone();
    let payload = match Payload::try_from((attrs.clone(), &mut input)) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error(),
    };
    if payload.attrs.is_ctx() {
        return quote! { #input };
    }
    let reflected = match codegen::Gen::generate(&payload) {
        Ok(p) => p,
        Err(err) => {
            return syn::Error::new_spanned(&input, err).to_compile_error();
        }
    };
    if let Err(err) = modificators::attrs::inject_repr_c(&mut input) {
        return syn::Error::new_spanned(&input, err).to_compile_error();
    }
    #[cfg(feature = "napi")]
    let napi_convert_impl = match codegen::generate_napi_impl(&payload_name, &payload_data) {
        Ok(tokens) => tokens,
        Err(err) => {
            return syn::Error::new_spanned(&input, err).to_compile_error();
        }
    };
    #[cfg(not(feature = "napi"))]
    let napi_convert_impl = quote! {};
    quote! {
        #input

        #napi_convert_impl
        #reflected
    }
}
