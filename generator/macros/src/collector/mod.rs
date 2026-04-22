mod blocks;
mod packet;
mod payloads;

use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(collector: &mut Collector, cfg: &Config) -> Result<TokenStream, E> {
    let pkg_name = get_pkg_name();
    let block = if collector.is_blocks_empty() {
        quote! {}
    } else {
        blocks::generate(
            collector
                .blocks
                .entry(pkg_name.clone())
                .or_default()
                .values()
                .collect::<Vec<&Block>>(),
            cfg,
        )?
    };
    let payload = if collector.is_payloads_empty() && cfg.is_no_default_payloads() {
        quote! {}
    } else {
        payloads::generate(
            collector
                .payloads
                .entry(pkg_name)
                .or_default()
                .values()
                .collect::<Vec<&Payload>>(),
            cfg,
        )?
    };
    let packet = if collector.is_blocks_empty()
        || (collector.is_payloads_empty() && cfg.is_no_default_payloads())
    {
        quote! {}
    } else {
        packet::generate()?
    };
    let output = quote! {
        #block
        #payload
        #packet
    };
    Ok(output)
}
