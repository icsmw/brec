use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
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
