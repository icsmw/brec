mod config;
mod parser;

use crate::Collector;
pub use config::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn generate(cfg: &Config) -> TokenStream {
    let mut collector = match Collector::get() {
        Ok(collector) => collector,
        Err(err) => return syn::Error::new(Span::call_site(), err).into_compile_error(),
    };
    if let Err(err) = collector.write(cfg) {
        return syn::Error::new(Span::call_site(), err).into_compile_error();
    }
    quote! {
        include!(concat!(env!("OUT_DIR"), "/brec.rs"));
    }
}
