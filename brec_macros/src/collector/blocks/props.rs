use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(blocks: &[&Block]) -> Result<TokenStream, E> {
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

pub fn gen_referred(blocks: &[&Block]) -> Result<TokenStream, E> {
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

pub fn referred_into(blocks: &[&Block]) -> Result<TokenStream, E> {
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

pub fn peek_as(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut impls = Vec::new();
    for blk in blocks.iter() {
        let block_name = blk.name();
        let fullname = blk.fullname()?;
        let referred_name = blk.referred_name();
        impls.push(quote! {
            impl<'a> brec::PeekAs<'a, #block_name> for BlockReferred<'a> {
                type Peeked = #referred_name<'a>;

                #[allow(unreachable_patterns)]
                fn peek_as(&'a self) -> Option<&'a Self::Peeked> {
                    match self {
                        BlockReferred::#fullname(inner) => Some(inner),
                        _ => None,
                    }
                }
            }
        });
    }
    Ok(quote! {
        #(#impls)*
    })
}
