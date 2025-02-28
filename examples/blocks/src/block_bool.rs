use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockBool {
    field: bool,
}

impl Arbitrary for BlockBool {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<bool>().prop_map(|field| BlockBool { field }).boxed()
    }
}
