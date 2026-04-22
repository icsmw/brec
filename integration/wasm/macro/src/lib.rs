use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives `brec::WasmConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `wasm` conversion
/// should be schema-driven and lossless for numeric edge cases.
///
/// See: <https://icsmw.github.io/brec/integrations/wasm/>
#[proc_macro_derive(Wasm)]
pub fn derive_wasm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match brec_in_wasm_gen::codegen::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}
