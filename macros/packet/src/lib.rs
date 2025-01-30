mod entities;
mod error;
mod modes;
mod parsing;
mod tokenized;

use entities::*;
use error::*;
use modes::*;
use tokenized::*;

use proc_macro as pm;
use proc_macro2 as pm2;
use quote::quote;
use std::convert::TryFrom;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Meta, Path};

fn parse(meta: Meta, mut input: DeriveInput) -> pm2::TokenStream {
    match meta {
        Meta::Path(path) => println!(">>>>>>>>>>>>>>>>>>>:{}", path.segments.len()),
        Meta::List(lt) => println!(">>>>>>>>>>>>>>>>>>> list: {}", lt.path.get_ident().unwrap()),
        Meta::NameValue(..) => println!(">>>>>>>>>>>>>>. name"),
    }
    let block = match Block::try_from(&mut input) {
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
    let meta: Meta = parse_quote! { block(mod_a::mod_b) };
    let input: DeriveInput = parse_quote! {
        #[block]
        struct MyBlock {
            field: u8,
            #[link_with(LogLevel)]
            log_level: u8,
        }
    };

    let expanded = parse(meta, input);
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

    println!(">>>>>>>>>>>>>>>>: {attrs:?}");
    // let input = parse_macro_input!(input as DeriveInput);

    // let struct_name = &input.ident;
    // let mut parsed = Vec::new();
    // if let Data::Struct(data_struct) = &input.data {
    //     if let Fields::Named(fields) = &data_struct.fields {
    //         for field in &fields.named {
    //             match Field::try_from(field) {
    //                 Ok(field) => parsed.push(field),
    //                 Err(err) => {
    //                     return err.to_compile_error().into();
    //                 }
    //             }
    //         }
    //     }
    // }
    // pm::TokenStream::from(quote! { #input })
    input
}
