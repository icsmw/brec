use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockI128 {
    field: i128,
}

impl Arbitrary for BlockI128 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i128>().prop_map(|field| BlockI128 { field }).boxed()
    }
}
