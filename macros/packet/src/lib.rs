mod collector;
mod entities;
mod error;
mod modes;
mod parsing;
mod tokenized;

pub(crate) use collector::*;
use entities::*;
use error::*;
use modes::*;
use tokenized::*;

use proc_macro as pm;
use proc_macro2 as pm2;
use quote::quote;
use std::convert::TryFrom;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Meta, Path};

fn parse(attrs: BlockAttrs, mut input: DeriveInput) -> pm2::TokenStream {
    let block = match Block::try_from((attrs, &mut input)) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error(),
    };
    let reflected = match modes::Structured::gen(&block) {
        Ok(p) => p,
        Err(err) => {
            return syn::Error::new_spanned(&input, err).to_compile_error();
        }
    };
    quote! {
        #input

        #reflected
    }
}

#[test]
fn test() {
    let input: DeriveInput = parse_quote! {
        #[block]
        struct MyBlock {
            field: u8,
            #[link_with(LogLevel)]
            log_level: u8,
        }
    };

    let expanded = parse(BlockAttrs::default(), input);
    let expected = quote! {
        struct MyBlock {
            field: u8,
            log_level: u8,
        }
    };

    assert_eq!(expanded.to_string(), expected.to_string());
}

#[proc_macro_attribute]
pub fn block(attr: pm::TokenStream, input: pm::TokenStream) -> pm::TokenStream {
    let attrs: BlockAttrs = parse_macro_input!(attr as BlockAttrs);
    let input = parse_macro_input!(input as DeriveInput);
    parse(attrs, input).into()
}
