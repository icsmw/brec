use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockI64 {
    field: i64,
}

impl Arbitrary for BlockI64 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i64>().prop_map(|field| BlockI64 { field }).boxed()
    }
}
