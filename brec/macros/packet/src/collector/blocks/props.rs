use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {#fullname(..) => #fullpath::ssize()});
    }
    Ok(quote! {
        impl brec::BlockDef for Block {}

        impl brec::Size for Block {
            fn size(&self) -> u64 {
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
        let referred_name = blk.referred_name();
        variants.push(quote! {#referred_name(..) => #fullpath::ssize()});
    }
    Ok(quote! {
        impl<'a> brec::BlockReferredDef<'a, Block> for BlockRef<'a> {}

        impl brec::Size for BlockRef<'_> {
            fn size(&self) -> u64 {
                match self {
                    #(#variants,)*
                }
            }
        }
    })
}
