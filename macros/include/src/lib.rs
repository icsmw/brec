use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn include_generated(_input: TokenStream) -> TokenStream {
    quote! {
        include!(concat!(env!("OUT_DIR"), "/brec.rs"));
    }
    .into()
}

//
