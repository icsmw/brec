use crate::*;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen(blocks: &[&Block], derives: Vec<String>) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    let derives = if derives.is_empty() {
        quote! {}
    } else {
        let ders = derives.into_iter().map(|der| format_ident!("{der}"));
        quote! {#[derive(#(#ders,)*)]}
    };
    Ok(quote! {
        #derives
        pub enum Block {
            #(#variants,)*
        }

        impl brec::BlockDef for Block {}
    })
}

pub fn gen_referred(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let referred_name = blk.referred_name();
        variants.push(quote! {#fullname(#referred_name<'a>)});
    }
    Ok(quote! {
        pub enum BlockReferred<'a> {
            #(#variants,)*
        }

        impl<'a> brec::BlockReferredDef<Block> for BlockReferred<'a> {}
    })
}
