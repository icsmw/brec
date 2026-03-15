use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct BlockBlob {
    blob: [u8; 100],
}

impl Arbitrary for BlockBlob {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop::collection::vec(any::<u8>(), 100)
            .prop_map(|v| {
                let mut blob = [0u8; 100];
                blob.copy_from_slice(&v);
                BlockBlob { blob }
            })
            .boxed()
    }
}
