use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn gen() -> Result<TokenStream, E> {
    Ok(quote! {
        pub type Packet = brec::PacketDef<Block, Payload, Payload>;

        pub type PacketBufReader<'a, R, W> =
            brec::PacketBufReaderDef<'a, R, W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type Rules<'a, W> = brec::RulesDef<W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type Rule<'a, W> = brec::RuleDef<W, Block, BlockReferred<'a>, Payload, Payload>;

        pub type RuleFnDef<D, S> = brec::RuleFnDef<D, S>;
    })
}
