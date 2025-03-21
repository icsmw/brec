use std::marker::PhantomData;

use crate::*;

pub struct PacketReferred<'a, B: BlockDef, BR: BlockReferredDef<B>> {
    pub blocks: Vec<BR>,
    pub header: PacketHeader,
    pub payload: Option<&'a [u8]>,
    _b: PhantomData<B>,
}

impl<'a, B: BlockDef, BR: BlockReferredDef<B>> PacketReferred<'a, B, BR> {
    pub fn new(blocks: Vec<BR>, header: PacketHeader) -> Self {
        Self {
            blocks,
            header,
            payload: None,
            _b: PhantomData,
        }
    }
}
