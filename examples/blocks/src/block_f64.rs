use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockF64 {
    field: f64,
}

impl Arbitrary for BlockF64 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<f64>().prop_map(|field| BlockF64 { field }).boxed()
    }
}
