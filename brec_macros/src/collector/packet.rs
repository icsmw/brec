use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn gen() -> Result<TokenStream, E> {
    let locked_storage = if cfg!(feature = "locked_storage") {
        quote! {
            #[allow(dead_code)]
            pub type FileStorage<'a> = brec::FileWriterDef<Block,  Payload, Payload>;
        }
    } else {
        quote! {}
    };
    Ok(quote! {
        #[allow(dead_code)]
        pub type Packet = brec::PacketDef<Block, Payload, Payload>;

        #[allow(dead_code)]
        pub type PacketBufReader<'a, R> =
            brec::PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rules<'a> = brec::RulesDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rule<'a> = brec::RuleDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type RuleFnDef<D, S> = brec::RuleFnDef<D, S>;

        #[allow(dead_code)]
        pub type Reader<'a, S> = brec::ReaderDef<S, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Writer<'a, S> = brec::WriterDef<S, Block, Payload, Payload>;

        #locked_storage
    })
}
