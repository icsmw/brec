use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct BlockBlobs {
    blob_a: [u8; 1],      // 1B
    blob_b: [u8; 1_024],  // 1Kb
    blob_c: [u8; 10_240], // 10Kb
}

impl Arbitrary for BlockBlobs {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            prop::collection::vec(any::<u8>(), 1),
            prop::collection::vec(any::<u8>(), 1_024),
            prop::collection::vec(any::<u8>(), 10_240),
        )
            .prop_map(|(a, b, c)| {
                let mut blob_a = [0u8; 1];
                blob_a.copy_from_slice(&a);
                let mut blob_b = [0u8; 1_024];
                blob_b.copy_from_slice(&b);
                let mut blob_c = [0u8; 10_240];
                blob_c.copy_from_slice(&c);
                BlockBlobs {
                    blob_a,
                    blob_b,
                    blob_c,
                }
            })
            .boxed()
    }
}
