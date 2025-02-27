use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockU128 {
    field: u128,
}

impl Arbitrary for BlockU128 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u128>().prop_map(|field| BlockU128 { field }).boxed()
    }
}
