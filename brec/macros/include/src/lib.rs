use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Path};

/// Inserts the generated glue code that connects user-defined `Block` and `Payload` types with the `brec` framework.
///
/// This macro must be called exactly once per crate and is responsible for:
///
/// - Implementing required `brec` traits for all user-defined `Block` types
/// - Implementing required `brec` traits for all user-defined `Payload` types
/// - Generating unified enums for blocks: `enum Block { ... }`
/// - Generating unified enums for payloads: `enum Payload { ... }`
/// - Exporting several convenience type aliases to simplify usage
///
/// ### Generated Aliases
/// The macro defines the following aliases to reduce verbosity when using `brec` types:
///
/// | Alias                    | Expanded to                                                                 |
/// |-------------------------|------------------------------------------------------------------------------|
/// | `Packet`                | `PacketDef<Block, Payload, Payload>`                                        |
/// | `PacketBufReader<'a, R>`| `PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>`     |
/// | `Rules<'a>`             | `RulesDef<Block, BlockReferred<'a>, Payload, Payload>`                      |
/// | `Rule<'a>`              | `RuleDef<Block, BlockReferred<'a>, Payload, Payload>`                       |
/// | `RuleFnDef<D, S>`       | `RuleFnDef<D, S>`                                                            |
/// | `Storage<S>`            | `StorageDef<S, Block, BlockReferred<'static>, Payload, Payload>`            |
///
/// These aliases make it easier to work with generated structures and remove the need to repeat generic parameters.
///
/// ---
///
/// ### Required Build Script
///
/// To enable this macro, you **must** include a `build.rs` file with the following content:
/// ```no_run
/// fn main() {
///     brec::build_setup();
/// }
/// ```
/// This step ensures the code generator runs during build and provides all required metadata.
///
/// ---
///
/// ### Usage Constraints
///
/// - The macro **must only be called once** per crate. Calling it more than once will result in compilation errors due to duplicate types and impls.
/// - The macro **must see all relevant types** (`Block`, `Payload`) in scope. You must ensure they are visible in the location where you call the macro.
///
/// ### Recommended Usage
///
/// Ensure that all blocks and payloads are imported at the point of macro usage:
/// ```no_run
/// pub use blocks::*;
/// pub use payloads::*;
///
/// brec::include_generated!();
/// ```
///
/// ### Alternate (not recommended) usage
///
/// If your module structure does not allow clean visibility, you may pass a path to be injected as a `use` statement:
/// ```no_run
/// brec::include_generated!("crate::*");
/// ```
/// This will implicitly add `use crate::*;` at the top of the generated code.  
/// This is **not recommended**, as it can lead to unclear dependencies and reduced maintainability.
///
/// ---
///
/// For more control, always make visibility explicit and organize your modules so the macro has access to all required types.
#[proc_macro]
pub fn include_generated(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        let import_path = if let Ok(path) = syn::parse::<Path>(input.clone()) {
            quote!(#path)
        } else {
            let cloned = input.clone();
            let path_tokens: proc_macro2::TokenStream =
                match parse_macro_input!(input as LitStr).value().parse() {
                    Ok(tk) => tk,
                    Err(err) => {
                        return syn::Error::new_spanned::<
                            &proc_macro2::TokenStream,
                            proc_macro2::LexError,
                        >(&cloned.into(), err)
                        .to_compile_error()
                        .into();
                    }
                };
            quote!(#path_tokens)
        };
        let out_dir = match std::env::var("OUT_DIR") {
            Ok(out_dir) => out_dir,
            Err(err) => {
                return syn::Error::new_spanned(&import_path, err)
                    .to_compile_error()
                    .into();
            }
        };
        let file_path = std::path::Path::new(&out_dir).join("brec.rs");
        if !file_path.exists() {
            return syn::Error::new_spanned(&import_path, "Output file brec.rs isn't generated")
                .to_compile_error()
                .into();
        }
        let old_content = match std::fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(err) => {
                return syn::Error::new_spanned(&import_path, err)
                    .to_compile_error()
                    .into();
            }
        };
        let new_content = format!("use {};\n\n{}", quote!(#import_path), old_content);

        if let Err(err) = std::fs::write(&file_path, new_content) {
            return syn::Error::new_spanned(&import_path, err)
                .to_compile_error()
                .into();
        }
    }
    quote! {
        include!(concat!(env!("OUT_DIR"), "/brec.rs"));
    }
    .into()
}
