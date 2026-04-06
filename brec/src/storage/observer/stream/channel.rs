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
    O: Send + Sync + 'static,
> {
    tx: mpsc::UnboundedSender<FileObserverEvent<B, P, Inner>>,
    _phantom: PhantomData<O>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    O: Send + Sync + 'static,
> SubscriptionDef<B, BR, P, Inner, O> for StreamSubscription<B, P, Inner, O>
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
    O: Send + Sync + 'static,
> {
    observer: FileObserverDef<B, BR, P, Inner, O>,
    pub(super) rx: mpsc::UnboundedReceiver<FileObserverEvent<B, P, Inner>>,
    _phantom: PhantomData<BR>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    O: Send + Sync + 'static,
> ObserverStreamState<B, BR, P, Inner, O>
where
    for<'a> Inner: PayloadSchema<Context<'a> = O>,
{
    /// Creates the internal observer and wires it to the channel-backed stream
    /// adapter.
    pub(super) fn new(path: impl AsRef<Path>, opt: O) -> Result<Self, Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::<B, P, Inner, O> {
            tx,
            _phantom: PhantomData,
        };
        let observer =
            FileObserverDef::with_opt(FileObserverOptions::new(path).subscribe(subscription), opt)?;
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

#[cfg(test)]
mod tests {
    use super::{ObserverStreamState, StreamSubscription};
    use crate::{
        DefaultPayloadContext, Error, FileObserverEvent, PacketDef, SubscriptionDef,
        SubscriptionErrorAction, SubscriptionUpdate,
        tests::{TestBlock, TestPayload},
    };
    use std::marker::PhantomData;
    use tempfile::NamedTempFile;
    use tokio::sync::mpsc;

    #[test]
    fn stream_subscription_maps_callbacks_to_events() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut sub = StreamSubscription::<TestBlock, TestPayload, TestPayload, DefaultPayloadContext> {
            tx,
            _phantom: PhantomData,
        };

        let update = <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_update(&mut sub, 7, 3);
        assert_eq!(update, SubscriptionUpdate::Read);
        assert!(
            matches!(
                rx.try_recv().expect("expected update event"),
                FileObserverEvent::Update { total: 7, added: 3 }
            ),
            "first event must be Update(total=7, added=3)"
        );

        let packet = PacketDef::<TestBlock, TestPayload, TestPayload>::default();
        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_packet(&mut sub, packet);
        assert!(
            matches!(
                rx.try_recv().expect("expected packet event"),
                FileObserverEvent::Packet(_)
            ),
            "second event must be Packet"
        );

        let action = <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_error(&mut sub, &Error::Test);
        assert_eq!(action, SubscriptionErrorAction::Continue);
        match rx.try_recv().expect("expected error event") {
            FileObserverEvent::Error(message) => assert!(
                !message.is_empty(),
                "error message forwarded to stream must not be empty"
            ),
            _ => panic!("third event must be Error"),
        }

        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_stopped(&mut sub, Some(Error::Test));
        assert!(
            matches!(
                rx.try_recv().expect("expected stopped event"),
                FileObserverEvent::Stopped(Some(Error::Test))
            ),
            "fourth event must be Stopped(Some(Error::Test))"
        );

        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_aborted(&mut sub);
        assert!(
            matches!(
                rx.try_recv().expect("expected aborted event"),
                FileObserverEvent::Aborted
            ),
            "fifth event must be Aborted"
        );
    }

    #[test]
    fn stream_subscription_swallows_send_errors() {
        let (tx, rx) = mpsc::unbounded_channel::<FileObserverEvent<TestBlock, TestPayload, TestPayload>>();
        drop(rx);

        let mut sub = StreamSubscription::<TestBlock, TestPayload, TestPayload, DefaultPayloadContext> {
            tx,
            _phantom: PhantomData,
        };

        assert_eq!(
            <StreamSubscription<
                TestBlock,
                TestPayload,
                TestPayload,
                DefaultPayloadContext,
            > as SubscriptionDef<
                TestBlock,
                TestBlock,
                TestPayload,
                TestPayload,
                DefaultPayloadContext,
            >>::on_update(&mut sub, 1, 1),
            SubscriptionUpdate::Read
        );
        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_packet(&mut sub, PacketDef::<TestBlock, TestPayload, TestPayload>::default());
        assert_eq!(
            <StreamSubscription<
                TestBlock,
                TestPayload,
                TestPayload,
                DefaultPayloadContext,
            > as SubscriptionDef<
                TestBlock,
                TestBlock,
                TestPayload,
                TestPayload,
                DefaultPayloadContext,
            >>::on_error(&mut sub, &Error::Test),
            SubscriptionErrorAction::Continue
        );
        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_stopped(&mut sub, None);
        <StreamSubscription<
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        > as SubscriptionDef<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >>::on_aborted(&mut sub);
    }

    #[tokio::test]
    async fn observer_stream_state_new_rejects_missing_file() {
        let missing = std::path::Path::new("/tmp/brec_missing_stream_observer_state_file.bin");
        let state = ObserverStreamState::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >::new(missing, DefaultPayloadContext::default());
        assert!(state.is_err());
    }

    #[tokio::test]
    async fn observer_stream_state_new_and_shutdown_on_existing_file() {
        let file = NamedTempFile::new().expect("temp file must be created");
        let mut state = ObserverStreamState::<
            TestBlock,
            TestBlock,
            TestPayload,
            TestPayload,
            DefaultPayloadContext,
        >::new(file.path(), DefaultPayloadContext::default())
        .expect("state must be created for existing file");

        state.shutdown().await;
    }
}
