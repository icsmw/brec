use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
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
