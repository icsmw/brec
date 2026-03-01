use tracing::debug;

use crate::*;

/// Defines the callback interface used by `FileObserverDef`.
///
/// A subscription receives successfully parsed packets, is notified about
/// observer errors, and can react to terminal lifecycle events.
pub trait SubscriptionDef<
    B: BlockDef + Send + 'static,
    BR: BlockReferredDef<B> + 'static,
    P: PayloadDef<Inner> + Send + 'static,
    Inner: PayloadInnerDef + Send + 'static,
>: Send
{
    /// Called when the observer detects updated storage state.
    ///
    /// - `total` is the total number of records currently available in storage.
    /// - `added` is the number of packets that became available since the last check.
    ///
    /// If this method returns `false`, newly available packets will not be
    /// read and `packet()` will not be called.
    ///
    /// If it returns `true`, `packet()` will be called once for each available
    /// packet.
    fn update(&mut self, total: usize, added: usize) -> bool;

    /// Called when a packet has been successfully read and parsed.
    ///
    /// This method is always preceded by a call to `update()`.
    /// It is not called if `update()` returns `false`.
    #[allow(unused)]
    fn packet(&mut self, packet: PacketDef<B, P, Inner>) {
        // default implementation
    }

    /// Called whenever the observer encounters an error.
    ///
    /// Some errors, such as failures related to source navigation or reading
    /// from the source, are fatal and will stop the observer regardless of the
    /// return value. In that case `stopped()` will be called.
    ///
    /// Errors related to packet parsing may be treated as non-fatal. Returning
    /// `false` allows the observer to ignore such an error and continue
    /// processing subsequent data.
    ///
    /// # Returns
    /// * `true` to stop the observer.
    /// * `false` to continue observing.
    fn err(&mut self, err: Error) -> bool {
        // default implementation
        debug!("Error on reading data with observer: {err}");
        false
    }

    /// Called when the observer stops normally.
    fn stopped(&mut self) {
        // default implementation
    }

    /// Called when the observer is aborted by an explicit shutdown signal.
    ///
    /// In this case `stopped()` is not called.
    fn aborted(&mut self) {
        // default implementation
    }
}
