use brec::prelude::*;
use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[block]
pub struct BlockF32 {
    field: f32,
}

impl Arbitrary for BlockF32 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<f32>().prop_map(|field| BlockF32 { field }).boxed()
    }
}
