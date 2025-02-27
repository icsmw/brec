use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockU16 {
    field: u16,
}

impl Arbitrary for BlockU16 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u16>().prop_map(|field| BlockU16 { field }).boxed()
    }
}
