mod subscription;

use crate::*;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub use subscription::*;

pub struct FileObserverOptions<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
    O: Send + Sync + 'static,
> {
    pub path: PathBuf,
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
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            subscription: None,
            _phantom: PhantomData,
        }
    }
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
