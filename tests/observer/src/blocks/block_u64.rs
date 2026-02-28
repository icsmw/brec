use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct BlockU64 {
    field: u64,
}

impl Arbitrary for BlockU64 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u64>().prop_map(|field| BlockU64 { field }).boxed()
    }
}
