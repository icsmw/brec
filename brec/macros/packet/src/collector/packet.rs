use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn gen() -> Result<TokenStream, E> {
    Ok(quote! {
        pub type Packet = brec::Packet<Block, Payload, Payload>;

        pub type PacketBufReader<'a, R, W> =
            brec::PacketBufReader<'a, R, W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type Rules<'a, W> = brec::Rules<W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type Rule<'a, W> = brec::Rule<W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type RuleFnDef<D, S> = brec::RuleFnDef<D, S>;
    })
}
