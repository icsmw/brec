use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockU16 {
    field: u16,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockU16 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<u16>().prop_map(|field| BlockU16 { field }).boxed()
    }
}
