mod subscription;

use crate::*;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub use subscription::*;

/// Builder options for creating [`FileObserverDef`](crate::storage::FileObserverDef).
pub struct FileObserverOptions<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
    O: Send + Sync + 'static,
> {
    /// Path to the observed storage file.
    pub path: PathBuf,
    /// Subscriber that receives observer callbacks.
    pub subscription: Option<S>,
    _phantom: PhantomData<(B, BR, P, Inner, O)>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
    O: Send + Sync + 'static,
> FileObserverOptions<B, BR, P, Inner, S, O>
{
    /// Creates observer options for a target file path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            subscription: None,
            _phantom: PhantomData,
        }
    }

    /// Attaches a subscription implementation used by the observer loop.
    pub fn subscribe(mut self, subscription: S) -> Self {
        self.subscription = Some(subscription);
        self
    }
}

impl<
    O: Send + Sync + 'static,
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
> std::fmt::Debug for FileObserverOptions<B, BR, P, Inner, S, O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileObserverOptions: {}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::FileObserverOptions;
    use crate::{
        DefaultPayloadContext,
        storage::observer::{SubscriptionDef, SubscriptionUpdate},
        tests::{TestBlock, TestPayload},
    };

    struct DummySubscription;

    impl SubscriptionDef<TestBlock, TestBlock, TestPayload, TestPayload, DefaultPayloadContext>
        for DummySubscription
    {
        fn on_update(&mut self, _total: usize, _added: usize) -> SubscriptionUpdate {
            SubscriptionUpdate::Skip
        }
    }

    #[test]
    fn file_observer_options_new_sets_path_and_empty_subscription() {
        let opts = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DummySubscription,
            DefaultPayloadContext,
        >::new("abc/storage.bin");
        assert!(opts.path.ends_with("abc/storage.bin"));
        assert!(opts.subscription.is_none());
    }

    #[test]
    fn file_observer_options_subscribe_sets_subscription() {
        let opts = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DummySubscription,
            DefaultPayloadContext,
        >::new("x.bin")
        .subscribe(DummySubscription);
        assert!(opts.subscription.is_some());
    }

    #[test]
    fn file_observer_options_debug_contains_path() {
        let opts = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DummySubscription,
            DefaultPayloadContext,
        >::new("dbg/path.bin");
        let dbg = format!("{opts:?}");
        assert!(dbg.contains("FileObserverOptions:"));
        assert!(dbg.contains("dbg/path.bin"));
    }
}
