use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {Block::#fullname(..) => #fullpath::ssize()});
    }
    Ok(quote! {
        impl brec::Size for Block {
            fn size(&self) -> u64 {
                use brec::StaticSize;
                match self {
                    #(#variants,)*
                }
            }
        }
    })
}

pub fn gen_referred(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullpath = blk.fullpath()?;
        let fullname = blk.fullname()?;
        variants.push(quote! {BlockReferred::#fullname(..) => #fullpath::ssize()});
    }
    Ok(quote! {
        impl brec::Size for BlockReferred<'_> {
            fn size(&self) -> u64 {
                use brec::StaticSize;
                match self {
                    #(#variants,)*
                }
            }
        }
    })
}

pub fn referred_into(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        variants.push(quote! {BlockReferred::#fullname(blk) => Block::#fullname(blk.into())});
    }
    Ok(quote! {
        impl std::convert::From<BlockReferred<'_>> for Block {
            fn from(blk: BlockReferred<'_>) -> Self {
                match blk {
                    #(#variants,)*
                }
            }
        }
    })
}
