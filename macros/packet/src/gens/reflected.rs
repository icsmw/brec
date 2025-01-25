use crate::*;

use proc_macro as pm;
use proc_macro2 as pm2;
use quote::{format_ident, quote};
use refs::*;
use std::borrow::Borrow;
use syn::{
    parse_macro_input, Fields, GenericArgument, Item, ItemStruct, PathArguments, ReturnType,
    Signature, Type, TypePath, TypeTuple,
};
pub fn serialize_name<S: AsRef<str>>(s: S) -> String {
    if s.as_ref().starts_with("r#") {
        s.as_ref().replace("r#", "")
    } else {
        s.as_ref().to_string()
    }
}

fn read_fields(fields: &Fields) -> Result<(), syn::Error> {
    let Fields::Named(ref fields) = fields else {
        return Err(syn::Error::new_spanned(fields, E::StructNotFound));
    };
    for field in fields.named.iter() {
        let name = field
            .ident
            .as_ref()
            .ok_or(syn::Error::new_spanned(field, E::StructNotFound))?;
    }
    Ok(())
    // if let Fields::Named(ref fields) = fields {
    //     for field in fields.named.iter() {
    //         let mut context = Context::try_from_or_default(&field.attrs)?;
    //         context.set_parent(parent_context.clone());
    //         if context.ignore_self() {
    //             continue;
    //         }
    //         let name = field.ident.clone().unwrap();
    //         parent.bind(Nature::Refered(Refered::Field(
    //             serialize_name(name.to_string()),
    //             context.clone(),
    //             Box::new(Nature::extract(&field.ty, context.clone(), cfg)?),
    //             context.get_bound(&name.to_string()),
    //         )))?;
    //     }
    //     parent.check_ignored_fields()?;
    // } else if let Fields::Unnamed(ref fields) = fields {
    //     if let Some(field) = fields.unnamed.first() {
    //         let mut context = Context::default();
    //         context.set_parent(parent_context.clone());
    //         parent.bind(Nature::Refered(Refered::Field(
    //             String::new(),
    //             context.clone(),
    //             Box::new(Nature::extract(&field.ty, context.clone(), cfg)?),
    //             None,
    //         )))?;
    //     }
    // } else {
    //     return Err(E::NotSupported(String::from("Unsupported type of fields")));
    // }
}
pub fn gen(strct: ItemStruct) -> Result<(), E> {
    let ItemStruct { ident, fields, .. } = strct;
    let name = ident.to_string();
    Ok(())
}
