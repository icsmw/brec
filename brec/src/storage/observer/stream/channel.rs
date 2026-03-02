use std::{marker::PhantomData, path::Path};

use tokio::sync::mpsc;

use crate::*;

use super::FileObserverEvent;

/// Internal subscription adapter that translates observer callbacks into stream
/// events sent through a Tokio channel.
///
/// This type is intentionally kept private to avoid exposing callback-oriented
/// details in the public stream API.
pub(super) struct StreamSubscription<
    B: BlockDef + Send + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> {
    tx: mpsc::UnboundedSender<FileObserverEvent<B, P, Inner>>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> SubscriptionDef<B, BR, P, Inner> for StreamSubscription<B, P, Inner>
{
    /// Always requests packet delivery and forwards the update event to the
    /// stream consumer.
    fn on_update(&mut self, total: usize, added: usize) -> SubscriptionUpdate {
        let _ = self.tx.send(FileObserverEvent::Update { total, added });
        SubscriptionUpdate::Read
    }

    /// Forwards parsed packets to the stream consumer.
    fn on_packet(&mut self, packet: PacketDef<B, P, Inner>) {
        let _ = self.tx.send(FileObserverEvent::Packet(packet));
    }

    /// Forwards non-terminal observer errors as text.
    fn on_error(&mut self, err: &Error) -> SubscriptionErrorAction {
        let _ = self.tx.send(FileObserverEvent::Error(err.to_string()));
        SubscriptionErrorAction::Continue
    }

    /// Forwards terminal stop information.
    fn on_stopped(&mut self, reason: Option<Error>) {
        let _ = self.tx.send(FileObserverEvent::Stopped(reason));
    }

    /// Forwards explicit observer abort.
    fn on_aborted(&mut self) {
        let _ = self.tx.send(FileObserverEvent::Aborted);
    }
}

/// Shared internal state used by the public stream facade.
///
/// This type owns:
/// - the underlying [`FileObserverDef`]
/// - the receiving side of the Tokio channel used for event delivery
///
/// The observer remains Tokio-backed, but the actual storage reads performed by
/// the observer are still synchronous and blocking at the file I/O layer.
pub(super) struct ObserverStreamState<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> {
    observer: FileObserverDef<B, BR, P, Inner>,
    pub(super) rx: mpsc::UnboundedReceiver<FileObserverEvent<B, P, Inner>>,
    _phantom: PhantomData<BR>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> ObserverStreamState<B, BR, P, Inner>
{
    /// Creates the internal observer and wires it to the channel-backed stream
    /// adapter.
    pub(super) fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::<B, P, Inner> { tx };
        let observer = FileObserverDef::new(FileObserverOptions::new(path).subscribe(subscription))?;
        Ok(Self {
            observer,
            rx,
            _phantom: PhantomData,
        })
    }

    /// Shuts down the underlying observer task.
    pub(super) async fn shutdown(&mut self) {
        self.observer.shutdown().await;
    }
}
