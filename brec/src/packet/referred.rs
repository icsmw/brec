use std::marker::PhantomData;

use crate::*;

pub struct PacketReferred<
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub blocks: Vec<BR>,
    pub header: PacketHeader,
    _b: PhantomData<B>,
    _p: PhantomData<P>,
    _i: PhantomData<Inner>,
}

impl<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
    PacketReferred<B, BR, P, Inner>
{
    pub fn new(blocks: Vec<BR>, header: PacketHeader) -> Self {
        Self {
            blocks,
            header,
            _b: PhantomData,
            _p: PhantomData,
            _i: PhantomData,
        }
    }
}
