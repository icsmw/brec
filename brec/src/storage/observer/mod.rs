mod sensor;

use crate::*;
use sensor::*;

pub struct ObserverDef<
    S: std::io::Read + std::io::Seek,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    reader: ReaderDef<S, B, BR, P, Inner>,
    sensor: Sensor,
}

impl<
        S: std::io::Read + std::io::Seek,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > ObserverDef<S, B, BR, P, Inner>
{
}
