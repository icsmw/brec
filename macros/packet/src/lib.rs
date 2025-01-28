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
use quote::{format_ident, quote};
use std::{borrow::Borrow, convert::TryFrom};
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericArgument, Item, ItemStruct,
    Meta, PathArguments, ReturnType, Signature, Type, TypePath, TypeTuple,
};

fn parse(input: DeriveInput) -> pm2::TokenStream {
    let packet = match Packet::try_from(&input) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error(),
    };
    let reflected = modes::StructuredMode::generate(&packet);
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

    // let opt: Opt = parse_macro_input!(args as Opt);
    // let item = parse_macro_input!(input as Item);
    // match item {
    //     Item::Struct(strc) => {}
    //     _ => {
    //         return syn::Error::new_spanned(item, E::StructNotFound)
    //             .to_compile_error()
    //             .into();
    //     }
    // }
    // let fn_name = &item_fn.sig.ident;
    // let args = &item_fn.sig.inputs;
    // let args_required: usize = args.len();
    // let mut arguments: Vec<pm2::TokenStream> = Vec::new();
    // let mut declarations: Vec<pm2::TokenStream> = Vec::new();
    // for (i, arg) in args.iter().enumerate() {
    //     if let syn::FnArg::Typed(pat_type) = arg {
    //         match pat_type.ty.borrow() {
    //             Type::Path(ty) => {
    //                 arguments.push(quote! {
    //                     args[#i].take().unwrap().value.try_to_rs().map_err(|err| LinkedErr::by_link(err, (&caller).into()))?,
    //                 });
    //                 match get_ty(&pat_type.ty) {
    //                     Ok(ty) => {
    //                         declarations.push(quote! { #ty.into() });
    //                     }
    //                     Err(err) => {
    //                         return syn::Error::new_spanned(pat_type, err)
    //                             .to_compile_error()
    //                             .into();
    //                     }
    //                 };
    //             }
    //             _ => {
    //                 return syn::Error::new_spanned(pat_type, "Only Type::Path are supported")
    //                     .to_compile_error()
    //                     .into();
    //             }
    //         }
    //     } else {
    //         return syn::Error::new_spanned(arg, "Method receivers are not supported")
    //             .to_compile_error()
    //             .into();
    //     }
    // }
    // let Some((type_ok, type_err)) = get_result_type(&item_fn.sig) else {
    //     return syn::Error::new_spanned(item_fn.sig, "Return type can be only Result<T,E>")
    //         .to_compile_error()
    //         .into();
    // };
    // let type_ok = match get_ty(type_ok) {
    //     Ok(ty) => ty,
    //     Err(err) => {
    //         return syn::Error::new_spanned(type_ok, err)
    //             .to_compile_error()
    //             .into();
    //     }
    // };
    // let func_name = format_ident!("{}_executor", fn_name);
    // let reference = if opt.ns.is_empty() {
    //     fn_name.to_string()
    // } else {
    //     format!("{}::{fn_name}", opt.ns)
    // };

    // pm::TokenStream::from(quote! {
    //     fn #func_name(args: Vec<FnArgValue>, _rt: Runtime, caller: SrcLink) -> RtPinnedResult<'static, LinkedErr<E>> {
    //         Box::pin(async move {
    //             if args.len() != #args_required {
    //                 return Err(LinkedErr::by_link(E::InvalidFnArgument, (&caller).into()));
    //             }
    //             #item_fn;
    //             let mut args = args
    //                 .into_iter()
    //                 .map(Some)
    //                 .collect::<Vec<Option<FnArgValue>>>();
    //             let result = #fn_name(#(#arguments)*)
    //                 .map_err(|err| LinkedErr::by_link(err, (&caller).into()))?;
    //             result.try_to_rtv().map_err(|err| LinkedErr::by_link(err, (&caller).into()))
    //         })
    //     }

    //     efns.add(
    //         #reference.to_string(),
    //         EmbeddedFnEntity {
    //             uuid: Uuid::new_v4(),
    //             fullname: #reference.to_string(),
    //             name: stringify!(#fn_name).to_string(),
    //             args: vec![#(#declarations,)*],
    //             result: #type_ok,
    //             exec: #func_name,
    //         },
    //     )?;
    // })
}

// let desc = ExecutorFnDescription::new(
//     #func_name,
//     vec![#(#declarations,)*],
//     #output_declaration
// );
// store.insert(
//     #reference,
//     desc,
// )?;
