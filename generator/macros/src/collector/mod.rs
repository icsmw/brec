mod blocks;
mod packet;
mod payloads;
mod scheme;

use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(collector: &mut Collector, cfg: &Config) -> Result<TokenStream, E> {
    let pkg_name = get_pkg_name();
    let blocks = collector
        .blocks
        .entry(pkg_name.clone())
        .or_default()
        .values()
        .cloned()
        .collect::<Vec<_>>();
    let payloads = collector
        .payloads
        .entry(pkg_name.clone())
        .or_default()
        .values()
        .cloned()
        .collect::<Vec<_>>();

    if cfg.is_scheme() {
        scheme::Scheme::generate(collector, cfg)?;
    }

    let block = if blocks.is_empty() {
        quote! {}
    } else {
        blocks::generate(blocks.iter().collect::<Vec<&Block>>(), cfg)?
    };
    let payload = if payloads.is_empty() && cfg.is_no_default_payloads() {
        quote! {}
    } else {
        payloads::generate(payloads.iter().collect::<Vec<&Payload>>(), cfg)?
    };
    let packet = if blocks.is_empty() || (payloads.is_empty() && cfg.is_no_default_payloads()) {
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
