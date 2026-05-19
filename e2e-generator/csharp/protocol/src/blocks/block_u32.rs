use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockU32 {
    field: u32,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockU32 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u32>().prop_map(|field| BlockU32 { field }).boxed()
    }
}
