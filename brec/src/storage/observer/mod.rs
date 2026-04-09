mod options;
mod sensor;
mod stream;

use std::marker::PhantomData;
use tokio::{
    select,
    task::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::*;
pub use options::*;
pub use sensor::*;
pub use stream::*;

/// Asynchronous observer that tails a storage file and forwards packets to a subscription.
pub struct FileObserverDef<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    O: Send + Sync + 'static,
> {
    handler: Option<JoinHandle<()>>,
    sd: CancellationToken,
    _phantom: PhantomData<(B, BR, P, Inner, O)>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    O: Send + Sync + 'static,
> FileObserverDef<B, BR, P, Inner, O>
where
    for<'a> Inner: PayloadSchema<Context<'a> = O>,
{
    /// Creates observer with explicit payload context options.
    pub fn with_opt<S>(
        mut options: FileObserverOptions<B, BR, P, Inner, S, O>,
        opt: O,
    ) -> Result<Self, Error>
    where
        S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
    {
        let Some(mut subscription) = options.subscription.take() else {
            return Err(Error::NoSubscription);
        };
        let sd = CancellationToken::new();
        let shutdown = sd.clone();
        let file = std::fs::File::open(&options.path)?;
        let mut reader: ReaderDef<std::fs::File, B, BR, P, Inner> =
            ReaderDef::new(file.try_clone()?)?;

        let (sensor, mut wake_rx) = Sensor::new(&options.path)?;

        let handler = task::spawn(async move {
            let mut opt = opt;
            let mut stop_reason: Option<Error> = None;
            let mut last = 0;
            let mut count = reader.count();
            if matches!(
                subscription.on_update(count, count),
                SubscriptionUpdate::Read
            ) {
                // Load first existed
                while let Some(pkg) = reader.iter(&mut opt).next() {
                    last += 1;
                    match pkg {
                        Ok(packet) => {
                            subscription.on_packet(packet);
                        }
                        Err(err) => {
                            if matches!(subscription.on_error(&err), SubscriptionErrorAction::Stop)
                            {
                                subscription.on_stopped(Some(err));
                                return;
                            }
                        }
                    }
                }
            }
            if shutdown.is_cancelled() {
                subscription.on_aborted();
                return;
            }
            select! {
                _ = shutdown.cancelled() => {
                    debug!("Cancel signal has been gotten");
                    subscription.on_aborted();
                }
                _ = async {
                    while wake_rx.recv().await.is_some() {
                        let added = match reader.reload() {
                            Ok(added) => added,
                            Err(Error::NotEnoughData(_)) => {
                                continue;
                            }
                            Err(err) => {
                                let _ = subscription.on_error(&err);
                                stop_reason = Some(err);
                                break;
                            }
                        };
                        if let Err(err) = sensor.processed(reader.get_offset()) {
                            let err = Error::from(err);
                            let _ = subscription.on_error(&err);
                            stop_reason = Some(err);
                            break;
                        }
                        if added == 0 {
                            continue;
                        }
                        count += added;
                        if !matches!(
                            subscription.on_update(count, added),
                            SubscriptionUpdate::Read
                        ) {
                            continue;
                        }
                        match reader.seek(last, &mut opt) {
                            Ok(mut iterator) => {
                                for pkg in iterator.by_ref() {
                                    last += 1;
                                    match pkg {
                                        Ok(packet) => {
                                            subscription.on_packet(packet);
                                        }
                                        Err(err) => {
                                            if matches!(
                                                subscription.on_error(&err),
                                                SubscriptionErrorAction::Stop
                                            ) {
                                                stop_reason = Some(err);
                                                break;
                                            }
                                        }
                                    }
                                }
                            },
                            Err(err) => {
                                let _ = subscription.on_error(&err);
                                stop_reason = Some(err);
                                break;
                            }
                        }
                        if stop_reason.is_some() {
                            break;
                        }
                    }
                    subscription.on_stopped(stop_reason);
                    drop(sensor);
                } => {
                    debug!("sensor loop is closed")
                }
            };
        });
        Ok(Self {
            handler: Some(handler),
            sd,
            _phantom: PhantomData,
        })
    }

    /// Creates observer using default payload context options.
    pub fn new<S>(options: FileObserverOptions<B, BR, P, Inner, S, O>) -> Result<Self, Error>
    where
        S: SubscriptionDef<B, BR, P, Inner, O> + 'static,
        O: Default,
    {
        Self::with_opt(options, O::default())
    }

    /// Requests graceful shutdown and waits for observer task completion.
    pub async fn shutdown(&mut self) {
        let Some(handler) = self.handler.take() else {
            return;
        };
        if !handler.is_finished() {
            self.sd.cancel();
        }
        let _ = handler.await;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DefaultPayloadContext, Error, FileObserverDef, FileObserverOptions, PacketDef,
        SubscriptionDef, SubscriptionErrorAction, SubscriptionUpdate,
        tests::{TestBlock, TestPayload},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tempfile::NamedTempFile;

    struct CountingSubscription {
        updates: Arc<AtomicUsize>,
        stopped: Arc<AtomicUsize>,
        aborted: Arc<AtomicUsize>,
    }

    impl SubscriptionDef<TestBlock, TestBlock, TestPayload, TestPayload, DefaultPayloadContext>
        for CountingSubscription
    {
        fn on_update(&mut self, _total: usize, _added: usize) -> SubscriptionUpdate {
            self.updates.fetch_add(1, Ordering::SeqCst);
            SubscriptionUpdate::Skip
        }

        fn on_packet(&mut self, _packet: PacketDef<TestBlock, TestPayload, TestPayload>) {}

        fn on_error(&mut self, _err: &Error) -> SubscriptionErrorAction {
            SubscriptionErrorAction::Continue
        }

        fn on_stopped(&mut self, _reason: Option<Error>) {
            self.stopped.fetch_add(1, Ordering::SeqCst);
        }

        fn on_aborted(&mut self) {
            self.aborted.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn counting_subscription_callbacks_update_counters_and_actions() {
        let updates = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicUsize::new(0));
        let aborted = Arc::new(AtomicUsize::new(0));

        let mut subscription = CountingSubscription {
            updates: updates.clone(),
            stopped: stopped.clone(),
            aborted: aborted.clone(),
        };

        assert!(matches!(
            subscription.on_update(10, 3),
            SubscriptionUpdate::Skip
        ));
        assert_eq!(updates.load(Ordering::SeqCst), 1);

        let action = subscription.on_error(&Error::Test);
        assert!(matches!(action, SubscriptionErrorAction::Continue));

        subscription.on_packet(PacketDef::default());
        subscription.on_stopped(None);
        subscription.on_aborted();

        assert_eq!(stopped.load(Ordering::SeqCst), 1);
        assert_eq!(aborted.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn observer_with_opt_returns_no_subscription_error() {
        let file = NamedTempFile::new().expect("temp file");
        let options = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            CountingSubscription,
            DefaultPayloadContext,
        >::new(file.path());

        let result = FileObserverDef::with_opt(options, ());
        assert!(matches!(result, Err(Error::NoSubscription)));
    }

    #[test]
    fn observer_with_opt_returns_io_error_for_missing_file() {
        let updates = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicUsize::new(0));
        let aborted = Arc::new(AtomicUsize::new(0));

        let subscription = CountingSubscription {
            updates,
            stopped,
            aborted,
        };

        let missing = "/tmp/brec-observer-missing-file-for-tests.bin";
        let options = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            CountingSubscription,
            DefaultPayloadContext,
        >::new(missing)
        .subscribe(subscription);

        let result = FileObserverDef::with_opt(options, ());
        assert!(matches!(result, Err(Error::Io(_))));
    }

    #[tokio::test]
    async fn observer_new_shutdown_is_idempotent_and_emits_lifecycle_callbacks() {
        let file = NamedTempFile::new().expect("temp file");
        let updates = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicUsize::new(0));
        let aborted = Arc::new(AtomicUsize::new(0));

        let subscription = CountingSubscription {
            updates: updates.clone(),
            stopped: stopped.clone(),
            aborted: aborted.clone(),
        };

        let options = FileObserverOptions::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            CountingSubscription,
            DefaultPayloadContext,
        >::new(file.path())
        .subscribe(subscription);

        let mut observer =
            FileObserverDef::new(options).expect("observer must be created for existing file");

        observer.shutdown().await;
        observer.shutdown().await;

        assert!(
            updates.load(Ordering::SeqCst) >= 1,
            "at least one on_update call is expected"
        );
        assert!(
            stopped.load(Ordering::SeqCst) + aborted.load(Ordering::SeqCst) >= 1,
            "observer should report terminal lifecycle callback"
        );
    }
}
