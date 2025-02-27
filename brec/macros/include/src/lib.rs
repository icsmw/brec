use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Path};

#[proc_macro]
pub fn include_generated(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        let import_path = if let Ok(path) = syn::parse::<Path>(input.clone()) {
            quote!(#path)
        } else {
            let path_tokens: proc_macro2::TokenStream = parse_macro_input!(input as LitStr)
                .value()
                .parse()
                .expect("Invalid Rust path");
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

//
