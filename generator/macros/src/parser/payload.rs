use proc_macro2::TokenStream;
use quote::quote;

use crate::*;
use std::convert::TryFrom;

pub fn parse(attrs: PayloadAttrs, mut input: DeriveInput) -> TokenStream {
    let payload_data = input.data.clone();
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
    let napi_convert_impl = {
        #[cfg(feature = "napi")]
        {
            match brec_in_node_gen::codegen::generate_impl(&payload_name, &payload_data) {
                Ok(tokens) => tokens,
                Err(err) => {
                    return syn::Error::new_spanned(&input, err).to_compile_error();
                }
            }
        }
        #[cfg(not(feature = "napi"))]
        {
            quote! {}
        }
    };
    let wasm_convert_impl = if cfg!(feature = "wasm") {
        match integrations::codegen::wasm::generate_impl(&payload_name, &payload_data) {
            Ok(tokens) => tokens,
            Err(err) => {
                return syn::Error::new_spanned(&input, err).to_compile_error();
            }
        }
    } else {
        quote! {}
    };
    let java_convert_impl = if cfg!(feature = "java") {
        match integrations::codegen::java::generate_impl(&payload_name, &payload_data) {
            Ok(tokens) => tokens,
            Err(err) => {
                return syn::Error::new_spanned(&input, err).to_compile_error();
            }
        }
    } else {
        quote! {}
    };
    let csharp_convert_impl = if cfg!(feature = "csharp") {
        match integrations::codegen::csharp::generate_impl(&payload_name, &payload_data) {
            Ok(tokens) => tokens,
            Err(err) => {
                return syn::Error::new_spanned(&input, err).to_compile_error();
            }
        }
    } else {
        quote! {}
    };
    quote! {
        #input

        #napi_convert_impl
        #wasm_convert_impl
        #java_convert_impl
        #csharp_convert_impl
        #reflected
    }
}
