use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockF32 {
    field: f32,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockF32 {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        any::<f32>()
            .prop_filter("no NaNs or infinite", |f| f.is_finite())
            .prop_map(|field| BlockF32 { field })
            .boxed()
    }
}
