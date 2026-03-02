use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn generate() -> Result<TokenStream, E> {
    let locked_storage = if cfg!(feature = "locked_storage") {
        quote! {
            #[allow(dead_code)]
            pub type FileStorage<'a> = brec::FileWriterDef<Block,  Payload, Payload>;
        }
    } else {
        quote! {}
    };
    let observer = if cfg!(feature = "observer") {
        quote! {
            #[allow(dead_code)]
            pub type SubscriptionUpdate = brec::SubscriptionUpdate;

            #[allow(dead_code)]
            pub type SubscriptionErrorAction = brec::SubscriptionErrorAction;

            #[allow(dead_code)]
            pub trait Subscription: Send + 'static {
                fn on_update(&mut self, total: usize, added: usize) -> SubscriptionUpdate;

                fn on_packet(&mut self, packet: Packet) {
                    let _ = packet;
                }

                fn on_error(&mut self, err: &brec::Error) -> SubscriptionErrorAction {
                    let _ = err;
                    SubscriptionErrorAction::Continue
                }

                fn on_stopped(&mut self, reason: Option<brec::Error>) {
                    let _ = reason;
                }

                fn on_aborted(&mut self) {}
            }

            #[allow(dead_code)]
            struct SubscriptionWrapper<S>(S);

            impl<S> brec::SubscriptionDef<Block, BlockReferred<'static>, Payload, Payload>
                for SubscriptionWrapper<S>
            where
                S: Subscription,
            {
                fn on_update(&mut self, total: usize, added: usize) -> brec::SubscriptionUpdate {
                    self.0.on_update(total, added)
                }

                fn on_packet(&mut self, packet: Packet) {
                    self.0.on_packet(packet)
                }

                fn on_error(&mut self, err: &brec::Error) -> brec::SubscriptionErrorAction {
                    self.0.on_error(err)
                }

                fn on_stopped(&mut self, reason: Option<brec::Error>) {
                    self.0.on_stopped(reason)
                }

                fn on_aborted(&mut self) {
                    self.0.on_aborted()
                }
            }

            #[allow(dead_code)]
            pub struct FileObserverOptions<S>
            where
                S: Subscription,
            {
                inner: brec::FileObserverOptions<
                    Block,
                    BlockReferred<'static>,
                    Payload,
                    Payload,
                    SubscriptionWrapper<S>,
                >,
            }

            impl<S> FileObserverOptions<S>
            where
                S: Subscription,
            {
                pub fn new(path: impl AsRef<std::path::Path>) -> Self {
                    Self {
                        inner: brec::FileObserverOptions::new(path),
                    }
                }

                pub fn subscribe(mut self, subscription: S) -> Self {
                    self.inner = self.inner.subscribe(SubscriptionWrapper(subscription));
                    self
                }
            }

            #[allow(dead_code)]
            pub struct FileObserver(
                brec::FileObserverDef<Block, BlockReferred<'static>, Payload, Payload>,
            );

            impl FileObserver {
                pub fn new<S>(options: FileObserverOptions<S>) -> Result<Self, brec::Error>
                where
                    S: Subscription,
                {
                    brec::FileObserverDef::new(options.inner).map(Self)
                }

                pub async fn shutdown(&mut self) {
                    self.0.shutdown().await
                }
            }

            #[allow(dead_code)]
            pub type FileObserverStream =
                brec::FileObserverStreamDef<Block, BlockReferred<'static>, Payload, Payload>;
        }
    } else {
        quote! {}
    };
    Ok(quote! {
        #[allow(dead_code)]
        pub type Packet = brec::PacketDef<Block, Payload, Payload>;

        #[allow(dead_code)]
        pub type BorrowedPacketBufReader<'a, R> =
            brec::PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type PacketBufReader<'a, R> = BorrowedPacketBufReader<'a, R>;

        #[allow(dead_code)]
        pub type PeekedBlocks<'a> = brec::PeekedBlocksDef<'a, BlockReferred<'a>>;

        #[allow(dead_code)]
        pub type PeekedBlock<'a> = brec::PeekedBlockDef<'a, BlockReferred<'a>>;

        #[allow(dead_code)]
        pub type BorrowedRules<'a> = brec::RulesDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rules<'a> = brec::RulesDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type BorrowedRule<'a> = brec::RuleDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Rule<'a> = brec::RuleDef<Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type RuleFnDef<D, S> = brec::RuleFnDef<D, S>;

        #[allow(dead_code)]
        pub type BorrowedReader<'a, S> =
            brec::ReaderDef<S, Block, BlockReferred<'a>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Reader<S> = brec::ReaderDef<S, Block, BlockReferred<'static>, Payload, Payload>;

        #[allow(dead_code)]
        pub type Writer<'a, S> = brec::WriterDef<S, Block, Payload, Payload>;

        #observer

        #locked_storage
    })
}
