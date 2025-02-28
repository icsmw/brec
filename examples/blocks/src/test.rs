use std::io::{BufReader, Cursor, Seek};

use brec::prelude::*;
use proptest::prelude::*;

use crate::*;

brec::include_generated!("crate::block_blob::*");

impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        println!(">>>>>>>>>>>>>>>>>>>>>>>>> ARDITARY");
        prop_oneof![
            BlockU8::arbitrary().prop_map(Block::BlockU8),
            // BlockU16::arbitrary_with(()).prop_map(Block::BlockU16),
            // BlockU32::arbitrary_with(()).prop_map(Block::BlockU32),
            // BlockU64::arbitrary_with(()).prop_map(Block::BlockU64),
            // BlockU128::arbitrary_with(()).prop_map(Block::BlockU128),
            // BlockI8::arbitrary_with(()).prop_map(Block::BlockI8),
            // BlockI16::arbitrary_with(()).prop_map(Block::BlockI16),
            // BlockI32::arbitrary_with(()).prop_map(Block::BlockI32),
            // BlockI64::arbitrary_with(()).prop_map(Block::BlockI64),
            // BlockI128::arbitrary_with(()).prop_map(Block::BlockI128),
            // BlockF32::arbitrary_with(()).prop_map(Block::BlockF32),
            // BlockF64::arbitrary_with(()).prop_map(Block::BlockF64),
            // BlockBool::arbitrary_with(()).prop_map(Block::BlockBool),
            // BlockBlob::arbitrary_with(()).prop_map(Block::BlockBlob),
            // BlockCombination::arbitrary_with(()).prop_map(Block::BlockCombination),
        ]
        .boxed()
    }
}

fn write_to_buf<W: std::io::Write>(buf: &mut W, blks: &[Block]) {
    for blk in blks.iter() {
        println!(
            "write: {} bytes",
            WriteTo::write(blk, buf).expect("Block is written")
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(5)
    })]


    #[test]
    fn test(blocks in proptest::collection::vec(Block::arbitrary(), 1..3)) {
        println!("created: {};", blocks.len());
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &blocks);
        let size = buf.len() as u64;
        println!("total size: {size}");
        let mut restored = Vec::new();
        let mut reader = BufReader::new(Cursor::new(buf));
        let mut consumed = 0;
        loop {
            match <Block as ReadBlockFrom>::read(&mut reader, false) {
                Ok(blk) => {
                    consumed = reader.stream_position().expect("Position is read");
                    restored.push(blk);
                }
                Err(err) => {
                    println!("{err}");
                    break;
                }
            }
        }
        assert_eq!(size, consumed);
        assert_eq!(blocks.len(), restored.len());
        for (left, right) in restored.iter().zip(blocks.iter()) {
            assert_eq!(left, right);
        }
    }

}
