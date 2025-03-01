use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockBlobs {
    blob_a: [u8; 1], // 1B
    blob_b: [u8; 1_024], // 1Kb
                     // blob_c: [u8; 1_048_576], // 1Mb
                     // blob_d: [u8; 1_073_741_824], // 1Gb
}

impl Arbitrary for BlockBlobs {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            prop::collection::vec(any::<u8>(), 1),
            prop::collection::vec(any::<u8>(), 1_024),
            // prop::collection::vec(any::<u8>(), 1_048_576),
            // prop::collection::vec(any::<u8>(), 1_073_741_824),
        )
            .prop_map(|(a, b)| {
                let mut blob_a = [0u8; 1];
                blob_a.copy_from_slice(&a);
                let mut blob_b = [0u8; 1_024];
                blob_b.copy_from_slice(&b);
                // let mut blob_c = [0u8; 1_048_576];
                // blob_c.copy_from_slice(&c);
                // let mut blob_d = [0u8; 1_073_741_824];
                // blob_d.copy_from_slice(&d);
                BlockBlobs {
                    blob_a,
                    blob_b,
                    // blob_c,
                    // blob_d,
                }
            })
            .boxed()
    }
}
