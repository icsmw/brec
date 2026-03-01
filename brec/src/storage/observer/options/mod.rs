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
    S: SubscriptionDef<B, BR, P, Inner> + 'static,
> {
    pub path: PathBuf,
    pub subscription: Option<S>,
    _phantom: PhantomData<(B, BR, P, Inner)>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner> + 'static,
> FileObserverOptions<B, BR, P, Inner, S>
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
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    S: SubscriptionDef<B, BR, P, Inner> + 'static,
> std::fmt::Debug for FileObserverOptions<B, BR, P, Inner, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileObserverOptions: {}", self.path.display())
    }
}
