use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct BlockI8 {
    field: i8,
}

impl Arbitrary for BlockI8 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i8>().prop_map(|field| BlockI8 { field }).boxed()
    }
}
