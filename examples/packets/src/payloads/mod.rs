#[cfg(test)]
mod a;
#[cfg(test)]
mod b;
#[cfg(test)]
mod c;

#[cfg(test)]
pub(crate) use a::*;
#[cfg(test)]
pub(crate) use b::*;
#[cfg(test)]
pub(crate) use c::*;

use crate::*;
use proptest::prelude::*;

impl Arbitrary for Payload {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        PayloadA::arbitrary().prop_map(Payload::PayloadA).boxed()
        //         prop_oneof![
        //     PayloadA::arbitrary().prop_map(Payload::PayloadA),
        //     PayloadB::arbitrary().prop_map(Payload::PayloadB),
        //     PayloadC::arbitrary().prop_map(Payload::PayloadC),
        // ]
        // .boxed()
    }
}
