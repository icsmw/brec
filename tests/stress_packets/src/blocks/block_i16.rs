use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct BlockI16 {
    field: i16,
}

impl Arbitrary for BlockI16 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i16>().prop_map(|field| BlockI16 { field }).boxed()
    }
}
