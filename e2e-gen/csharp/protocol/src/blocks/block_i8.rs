use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockI8 {
    field: i8,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockI8 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<i8>().prop_map(|field| BlockI8 { field }).boxed()
    }
}
