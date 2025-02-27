use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockI32 {
    field: i32,
}

impl Arbitrary for BlockI32 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i32>().prop_map(|field| BlockI32 { field }).boxed()
    }
}
