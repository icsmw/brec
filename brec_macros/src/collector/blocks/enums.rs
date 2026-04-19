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
    let napi_impl = if cfg!(feature = "napi") {
        integrations::collector::blocks::napi::generate_impl(blocks)?
    } else {
        quote! {}
    };
    let wasm_impl = if cfg!(feature = "wasm") {
        integrations::collector::blocks::wasm::generate_impl(blocks)?
    } else {
        quote! {}
    };
    let java_impl = if cfg!(feature = "java") {
        integrations::collector::blocks::java::generate_impl(blocks)?
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
