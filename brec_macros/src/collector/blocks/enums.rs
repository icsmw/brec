use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(blocks: &[&Block], derives: Vec<TokenStream>, _cfg: &Config) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    let derives = if derives.is_empty() {
        quote! {}
    } else {
        quote! {#[derive(#(#derives,)*)]}
    };
    Ok(quote! {
        #derives
        #[allow(non_snake_case)]
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
