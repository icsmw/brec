use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn gen() -> Result<TokenStream, E> {
    Ok(quote! {
        #[allow(dead_code)]
        pub type Packet = brec::PacketDef<Block, Payload, Payload>;

        #[allow(dead_code)]
        pub type PacketBufReader<'a, R, W> =
            brec::PacketBufReaderDef<'a, R, W, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rules<'a, W> = brec::RulesDef<W, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rule<'a, W> = brec::RuleDef<W, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type RuleFnDef<D, S> = brec::RuleFnDef<D, S>;

        #[allow(dead_code)]
        pub type Storage<S> = brec::StorageDef<S, Block, Payload, Payload>;
    })
}
