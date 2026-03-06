mod subscription;

use crate::*;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub use subscription::*;

pub struct FileObserverOptions<
    O: Default + Send + 'static,
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<O, Inner> + Send + 'static,
    Inner: PayloadInnerDef<O> + Send + 'static,
    S: SubscriptionDef<O, B, BR, P, Inner> + 'static,
> {
    pub path: PathBuf,
    pub subscription: Option<S>,
    _phantom: PhantomData<(B, BR, P, Inner, O)>,
}

impl<
    O: Default + Send + 'static,
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<O, Inner> + Send + 'static,
    Inner: PayloadInnerDef<O> + Send + 'static,
    S: SubscriptionDef<O, B, BR, P, Inner> + 'static,
> FileObserverOptions<O, B, BR, P, Inner, S>
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
    O: Default + Send + 'static,
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<O, Inner> + Send + 'static,
    Inner: PayloadInnerDef<O> + Send + 'static,
    S: SubscriptionDef<O, B, BR, P, Inner> + 'static,
> std::fmt::Debug for FileObserverOptions<O, B, BR, P, Inner, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileObserverOptions: {}", self.path.display())
    }
}
