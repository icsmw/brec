use std::io::{BufReader, Cursor, Seek};

use brec::prelude::*;
use proptest::prelude::*;

use crate::*;

brec::include_generated!("crate::*");

impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            BlockU8::arbitrary().prop_map(Block::BlockU8),
            BlockU16::arbitrary().prop_map(Block::BlockU16),
            BlockU32::arbitrary().prop_map(Block::BlockU32),
            BlockU64::arbitrary().prop_map(Block::BlockU64),
            BlockU128::arbitrary().prop_map(Block::BlockU128),
            BlockI8::arbitrary().prop_map(Block::BlockI8),
            BlockI16::arbitrary().prop_map(Block::BlockI16),
            BlockI32::arbitrary().prop_map(Block::BlockI32),
            BlockI64::arbitrary().prop_map(Block::BlockI64),
            BlockI128::arbitrary().prop_map(Block::BlockI128),
            BlockF32::arbitrary().prop_map(Block::BlockF32),
            BlockF64::arbitrary().prop_map(Block::BlockF64),
            BlockBool::arbitrary().prop_map(Block::BlockBool),
            BlockBlob::arbitrary().prop_map(Block::BlockBlob),
            BlockCombination::arbitrary().prop_map(Block::BlockCombination),
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
        ..ProptestConfig::with_cases(100)
    })]


    #[test]
    fn test(blks in proptest::collection::vec(any::<Block>(), 1..100)) {
        println!("created: {};", blks.len());
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &blks);
        let size = buf.len() as u64;
        let mut restored = Vec::new();
        let mut reader = BufReader::new(Cursor::new(buf));
        let mut consumed = 0;
        println!("start reading from total size: {size}");
        loop {
            match <Block as TryReadFrom>::try_read(&mut reader) {
                Ok(ReadStatus::Success(blk)) => {
                    consumed = reader.stream_position().expect("Position is read");
                    restored.push(blk);
                    println!("consumed: {consumed}");
                },
                Ok(ReadStatus::NotEnoughData(n)) => {
                    println!("NotEnoughData: {n}");
                    break;
                }
                Err(err) => {
                    println!("Fail to read: {err}");
                    break;
                }
            }
        }
        assert_eq!(size, consumed);
        assert_eq!(blks.len(), restored.len());
        for (left, right) in restored.iter().zip(blks.iter()) {
            // if let (Block::BlockCombination(left), Block::BlockCombination(right)) = (left, right) {
            //     assert_eq!(left.field_u8, right.field_u8);
            //     assert_eq!(left.field_u16, right.field_u16);
            //     assert_eq!(left.field_u32, right.field_u32);
            //     assert_eq!(left.field_u64, right.field_u64);
            //     assert_eq!(left.field_u128, right.field_u128);
            //     assert_eq!(left.field_i8, right.field_i8);
            //     assert_eq!(left.field_i16, right.field_i16);
            //     assert_eq!(left.field_i32, right.field_i32);
            //     assert_eq!(left.field_i64, right.field_i64);
            //     assert_eq!(left.field_i128, right.field_i128);
            //     assert_eq!(left.field_f32, right.field_f32);
            //     assert_eq!(left.field_f64, right.field_f64);
            //     assert_eq!(left.field_bool, right.field_bool);
            //     assert_eq!(left.blob_a.len(), right.blob_a.len());
            //     assert_eq!(left.blob_b.len(), right.blob_b.len());
            //     assert_eq!(left.blob_c.len(), right.blob_c.len());
            //     assert_eq!(left.blob_d.len(), right.blob_d.len());
            // } else {
            //     assert_eq!(left, right);
            // }
            assert_eq!(left, right);
        }
    }

}
