use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockU32 {
    field: u32,
}

impl Arbitrary for BlockU32 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u32>().prop_map(|field| BlockU32 { field }).boxed()
    }
}
