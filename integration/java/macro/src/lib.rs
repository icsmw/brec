use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives `brec::JavaConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `java` conversion
/// should be schema-driven for JNI-backed integrations.
///
/// See: <https://icsmw.github.io/brec/integrations/java/>
#[proc_macro_derive(Java)]
pub fn derive_java(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match brec_in_java_gen::codegen::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}
