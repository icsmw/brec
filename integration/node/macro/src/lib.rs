use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives `brec::NapiConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `napi` conversion
/// should be schema-driven and lossless for numeric edge cases.
///
/// See: <https://icsmw.github.io/brec/integrations/napi/>
#[proc_macro_derive(Napi)]
pub fn derive_napi(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match brec_node_gen::codegen::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}
