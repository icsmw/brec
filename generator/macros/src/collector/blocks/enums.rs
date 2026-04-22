use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(
    blocks: &[&Block],
    derives: Vec<TokenStream>,
    _cfg: &Config,
) -> Result<TokenStream, E> {
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
    let napi_impl = {
        #[cfg(feature = "napi")]
        {
            brec_in_node_gen::collector::block::generate_impl(blocks)?
        }
        #[cfg(not(feature = "napi"))]
        {
            quote! {}
        }
    };
    let wasm_impl = {
        #[cfg(feature = "wasm")]
        {
            brec_in_wasm_gen::collector::block::generate_impl(blocks)?
        }
        #[cfg(not(feature = "wasm"))]
        {
            quote! {}
        }
    };
    let java_impl = if cfg!(feature = "java") {
        integrations::collector::blocks::java::generate_impl(blocks)?
    } else {
        quote! {}
    };
    let csharp_impl = if cfg!(feature = "csharp") {
        integrations::collector::blocks::csharp::generate_impl(blocks)?
    } else {
        quote! {}
    };
    Ok(quote! {
        #derives
        #[allow(non_snake_case)]
        pub enum Block {
            #(#variants,)*
        }

        impl brec::BlockDef for Block {}
        #napi_impl
        #wasm_impl
        #java_impl
        #csharp_impl
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
