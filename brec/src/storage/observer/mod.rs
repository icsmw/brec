mod options;
mod sensor;

use std::marker::PhantomData;
use tokio::{
    select,
    task::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, instrument};

use crate::*;
pub use options::*;
pub use sensor::*;

pub struct FileObserverDef<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> {
    handler: Option<JoinHandle<()>>,
    sd: CancellationToken,
    _phantom: PhantomData<(B, BR, P, Inner)>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> FileObserverDef<B, BR, P, Inner>
{
    #[instrument]
    pub fn new<S>(mut options: FileObserverOptions<B, BR, P, Inner, S>) -> Result<Self, Error>
    where
        S: SubscriptionDef<B, BR, P, Inner> + 'static,
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
            let mut stop_reason: Option<Error> = None;
            let mut last = 0;
            let mut count = reader.count();
            if matches!(
                subscription.on_update(count, count),
                SubscriptionUpdate::Read
            ) {
                // Load first existed
                while let Some(pkg) = reader.iter().next() {
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
                        match reader.seek(last + 1) {
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
