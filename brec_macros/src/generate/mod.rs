mod config;
mod parser;

use crate::Collector;
pub use config::*;
use proc_macro2::{Span, TokenStream};

pub fn generate(cfg: &Config) -> TokenStream {
    let mut collector = match Collector::get() {
        Ok(collector) => collector,
        Err(err) => return syn::Error::new(Span::call_site(), err).into_compile_error(),
    };
    match collector.render(cfg) {
        Ok(tokens) => tokens,
        Err(err) => syn::Error::new(Span::call_site(), err).into_compile_error(),
    }
}
