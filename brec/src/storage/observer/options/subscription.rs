use tracing::debug;

use crate::*;

/// Controls how the observer should handle newly detected packets after `on_update`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionUpdate {
    /// Skip reading newly available packets for this update cycle.
    Skip,
    /// Read newly available packets and deliver them via `on_packet`.
    Read,
}

/// Controls how the observer should react after `on_error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionErrorAction {
    /// Keep observing and continue processing subsequent updates.
    Continue,
    /// Stop the observer.
    Stop,
}

/// Defines the callback interface used by `FileObserverDef`.
///
/// A subscription receives successfully parsed packets, is notified about
/// observer errors, and can react to terminal lifecycle events.
pub trait SubscriptionDef<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
    O: Send + Sync + 'static,
>: Send
{
    /// Called when the observer detects updated storage state.
    ///
    /// - `total` is the total number of records currently available in storage.
    /// - `added` is the number of packets that became available since the last check.
    ///
    /// If this method returns [`SubscriptionUpdate::Skip`], newly available
    /// packets will not be read and `on_packet()` will not be called.
    ///
    /// If it returns [`SubscriptionUpdate::Read`], `on_packet()` will be called
    /// once for each available packet.
    fn on_update(&mut self, total: usize, added: usize) -> SubscriptionUpdate;

    /// Called when a packet has been successfully read and parsed.
    ///
    /// This method is always preceded by a call to `on_update()`.
    /// It is not called if `on_update()` returns [`SubscriptionUpdate::Skip`].
    #[allow(unused)]
    fn on_packet(&mut self, packet: PacketDef<B, P, Inner>) {
        // default implementation
        let _ = packet;
    }

    /// Called whenever the observer encounters an error.
    ///
    /// Some errors, such as failures related to source navigation or reading
    /// from the source, are fatal and will stop the observer regardless of the
    /// return value. In that case `stopped()` will be called.
    ///
    /// Errors related to packet parsing may be treated as non-fatal. Returning
    /// [`SubscriptionErrorAction::Continue`] allows the observer to ignore such
    /// an error and continue processing subsequent data.
    ///
    /// The error is passed by reference so the same terminal error can later be
    /// reported again via `on_stopped(Some(...))` when the observer cannot
    /// continue.
    ///
    /// # Returns
    /// * [`SubscriptionErrorAction::Stop`] to stop the observer.
    /// * [`SubscriptionErrorAction::Continue`] to continue observing.
    fn on_error(&mut self, err: &Error) -> SubscriptionErrorAction {
        // default implementation
        debug!("Error on reading data with observer: {err}");
        SubscriptionErrorAction::Continue
    }

    /// Called when the observer stops.
    ///
    /// `reason` is `None` for a normal stop and `Some(error)` when the observer
    /// cannot continue due to a terminal failure.
    fn on_stopped(&mut self, reason: Option<Error>) {
        // default implementation
        let _ = reason;
    }

    /// Called when the observer is aborted by an explicit shutdown signal.
    ///
    /// In this case `stopped()` is not called.
    fn on_aborted(&mut self) {
        // default implementation
    }
}
