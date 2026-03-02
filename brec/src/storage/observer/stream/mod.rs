mod channel;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::*;

use self::channel::ObserverStreamState;

/// Events emitted by [`FileObserverStreamDef`].
///
/// The stream mirrors the observer lifecycle:
/// - storage growth is reported via [`FileObserverEvent::Update`]
/// - successfully parsed packets are emitted via [`FileObserverEvent::Packet`]
/// - non-terminal observer errors are forwarded as [`FileObserverEvent::Error`]
/// - terminal completion is reported via [`FileObserverEvent::Stopped`] or
///   [`FileObserverEvent::Aborted`]
///
/// This event stream is intentionally explicit instead of yielding only packets,
/// because the underlying observer has meaningful lifecycle transitions that
/// callers often need to react to.
pub enum FileObserverEvent<
    B: BlockDef + Send + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> {
    /// Storage metadata changed.
    ///
    /// `total` is the total number of packets currently visible in storage,
    /// `added` is the number of packets discovered since the previous update.
    Update { total: usize, added: usize },

    /// A packet was successfully read and parsed.
    Packet(PacketDef<B, P, Inner>),

    /// A non-terminal observer error.
    ///
    /// At this layer the error is exposed as text because `SubscriptionDef`
    /// reports it by shared reference, so ownership of the original [`Error`]
    /// is not available.
    Error(String),

    /// The observer stopped.
    ///
    /// `None` means a normal stop.
    /// `Some(error)` means the observer could not continue because of a
    /// terminal failure.
    Stopped(Option<Error>),

    /// The observer was explicitly aborted by shutdown.
    Aborted,
}

/// Tokio-backed observer stream.
///
/// This is a convenience facade over [`FileObserverDef`] that converts observer
/// callbacks into a `tokio_stream::Stream` of [`FileObserverEvent`] values.
///
/// # Runtime Model
///
/// This type is async at the orchestration layer:
/// - wakeups are delivered asynchronously
/// - observer lifecycle is driven by Tokio tasks
/// - events are forwarded through an async-aware channel
///
/// However, this does **not** mean the storage access itself is non-blocking.
/// At the low level the observer still relies on synchronous, blocking file I/O
/// (`std::fs::File`, `Read`, `Seek`) through the storage reader. In other words:
///
/// - async control flow: yes
/// - blocking file access under the hood: also yes
///
/// This is acceptable for the current observer design because the async layer is
/// mainly about coordination and integration with Tokio-based applications, not
/// about turning regular file I/O into true non-blocking disk access.
pub struct FileObserverStreamDef<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> {
    inner: ObserverStreamState<B, BR, P, Inner>,
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> Unpin for FileObserverStreamDef<B, BR, P, Inner>
{
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> FileObserverStreamDef<B, BR, P, Inner>
{
    /// Creates a new observer stream for the target storage file.
    ///
    /// The observer starts immediately and begins emitting lifecycle events as
    /// soon as storage changes are detected.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        Ok(Self {
            inner: ObserverStreamState::new(path)?,
        })
    }

    /// Stops the underlying observer task and waits for it to finish.
    ///
    /// After shutdown, no more events will be produced.
    pub async fn shutdown(&mut self) {
        self.inner.shutdown().await;
    }
}

impl<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
> tokio_stream::Stream for FileObserverStreamDef<B, BR, P, Inner>
{
    type Item = FileObserverEvent<B, P, Inner>;

    /// Polls the next observer event from the internal Tokio channel.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner.rx).poll_recv(cx)
    }
}
