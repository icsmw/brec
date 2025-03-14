#[cfg(test)]
mod tests;

mod codegen;
mod collector;
mod entities;
mod error;
mod modificators;
mod parser;
mod parsing;
mod tokenized;

use codegen::*;
use collector::*;
use entities::*;
use error::*;
use tokenized::*;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn block(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as BlockAttrs);
    let input = parse_macro_input!(input as DeriveInput);
    parser::block::parse(attrs, input).into()
}

#[proc_macro_attribute]
pub fn payload(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as PayloadAttrs);
    let input = parse_macro_input!(input as DeriveInput);
    parser::payload::parse(attrs, input).into()
}
