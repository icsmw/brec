use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
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
