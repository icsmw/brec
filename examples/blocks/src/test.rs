use brec::prelude::*;
use proptest::prelude::*;

use crate::block_blob::*;

brec::include_generated!("crate::block_blob::*");

// impl Arbitrary for Block {
//     type Parameters = ();

//     type Strategy = BoxedStrategy<Self>;

//     fn arbitrary_with(_: ()) -> Self::Strategy {
//         prop::collection::vec(any::<u8>(), 100)
//             .prop_map(|v| {
//                 let mut blob = [0u8; 100];
//                 blob.copy_from_slice(&v);
//                 BlockBlob { blob }
//             })
//             .boxed()
//     }
// }
