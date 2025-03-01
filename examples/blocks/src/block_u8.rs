use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BlockU8 {
    field: u8,
}

impl Arbitrary for BlockU8 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u8>().prop_map(|field| BlockU8 { field }).boxed()
    }
}
