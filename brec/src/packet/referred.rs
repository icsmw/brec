use std::marker::PhantomData;

use crate::*;

/// A lightweight representation of a parsed packet with block references and optional raw payload.
///
/// This structure is used when:
/// - You want to keep references to already-decoded blocks (`BR`)
/// - You don’t need to immediately decode the payload (it can remain as a raw slice)
///
/// Useful in zero-copy parsing scenarios or when working with external buffers.
///
/// # Type Parameters
/// - `B`: The original block type, implementing [`BlockDef`].
/// - `BR`: The referred/parsed block type, implementing [`BlockReferredDef<B>`].
///
/// # Fields
/// - `blocks`: A vector of referred block objects.
/// - `header`: The parsed packet header.
/// - `payload`: Optional raw payload slice (usually borrowed from a buffer).
/// - `_b`: Phantom marker to retain the `B` type.
pub struct PacketReferred<'a, B: BlockDef, BR: BlockReferredDef<B>> {
    pub blocks: Vec<BR>,
    pub header: PacketHeader,
    pub payload: Option<&'a [u8]>,
    _b: PhantomData<B>,
}

impl<B: BlockDef, BR: BlockReferredDef<B>> PacketReferred<'_, B, BR> {
    /// Constructs a new `PacketReferred` with the given blocks and header.
    ///
    /// The payload is not set by default and must be attached manually if needed.
    ///
    /// # Arguments
    /// - `blocks` – A list of parsed or referred blocks.
    /// - `header` – The parsed packet header associated with the packet.
    ///
    /// # Returns
    /// A new `PacketReferred` instance with `payload = None`.
    pub fn new(blocks: Vec<BR>, header: PacketHeader) -> Self {
        Self {
            blocks,
            header,
            payload: None,
            _b: PhantomData,
        }
    }
}
