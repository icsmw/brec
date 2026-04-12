mod a;
mod b;
mod c;
mod d;

pub use a::*;
pub use b::*;
pub use c::*;
pub use d::*;

#[cfg(any(test, feature = "test-utils"))]
use crate::*;
#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for Payload {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            PayloadA::arbitrary().prop_map(Payload::PayloadA),
            PayloadB::arbitrary().prop_map(Payload::PayloadB),
            PayloadC::arbitrary().prop_map(Payload::PayloadC),
            PayloadD::arbitrary().prop_map(Payload::PayloadD),
        ]
        .boxed()
    }
}
