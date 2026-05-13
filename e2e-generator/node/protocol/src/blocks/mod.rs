mod block_bool;
mod block_comb;
mod block_enums;
mod block_f32;
mod block_f64;
mod block_i128;
mod block_i16;
mod block_i32;
mod block_i64;
mod block_i8;
mod block_u16;
mod block_u128;
mod block_u32;
mod block_u64;
mod block_u8;

pub use block_bool::*;
pub use block_comb::*;
pub use block_enums::*;
pub use block_f32::*;
pub use block_f64::*;
pub use block_i128::*;
pub use block_i16::*;
pub use block_i32::*;
pub use block_i64::*;
pub use block_i8::*;
pub use block_u16::*;
pub use block_u128::*;
pub use block_u32::*;
pub use block_u64::*;
pub use block_u8::*;

#[cfg(any(test, feature = "test-utils"))]
use crate::*;
#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            BlockU8::arbitrary().prop_map(Block::BlockU8),
            BlockU16::arbitrary().prop_map(Block::BlockU16),
            BlockU32::arbitrary().prop_map(Block::BlockU32),
            BlockI8::arbitrary().prop_map(Block::BlockI8),
            BlockI16::arbitrary().prop_map(Block::BlockI16),
            BlockI32::arbitrary().prop_map(Block::BlockI32),
            BlockI64::arbitrary().prop_map(Block::BlockI64),
            BlockI128::arbitrary().prop_map(Block::BlockI128),
            BlockF32::arbitrary().prop_map(Block::BlockF32),
            BlockF64::arbitrary().prop_map(Block::BlockF64),
            BlockU64::arbitrary().prop_map(Block::BlockU64),
            BlockU128::arbitrary().prop_map(Block::BlockU128),
            BlockBool::arbitrary().prop_map(Block::BlockBool),
            BlockEnums::arbitrary().prop_map(Block::BlockEnums),
            BlockCombination::arbitrary().prop_map(Block::BlockCombination),
        ]
        .boxed()
    }
}
