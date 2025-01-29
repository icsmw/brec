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
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

fn parse(input: DeriveInput) -> pm2::TokenStream {
    let packet = match Packet::try_from(&input) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error(),
    };
    let reflected = modes::Structured::gen(&packet);
    quote! {
        #input

        #reflected
    }
}

#[test]
fn test() {
    let input: DeriveInput = parse_quote! {
        #[packet]
        struct MyPacket {
            field: u8,
            #[link_with(LogLevel)]
            log_level: u8,
        }
    };

    let expanded = parse(input);
    let expected = quote! {
        struct MyPacket {
            field: u8,
            log_level: u8,
        }
    };

    assert_eq!(expanded.to_string(), expected.to_string());
}

#[proc_macro_attribute]
pub fn packet(args: pm::TokenStream, input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let mut parsed = Vec::new();
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                match Field::try_from(field) {
                    Ok(field) => parsed.push(field),
                    Err(err) => {
                        return err.to_compile_error().into();
                    }
                }
            }
        }
    }
    pm::TokenStream::from(quote! { #input })
}
