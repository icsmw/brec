use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {#fullname(#fullpath)});
    }
    Ok(quote! {
        pub enum Block {
            #(#variants,)*
        }
    })
}

pub fn gen_referred(blocks: &[Block]) -> Result<TokenStream, E> {
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
    })
}
