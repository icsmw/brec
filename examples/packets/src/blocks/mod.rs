#[cfg(test)]
mod block_blob;
#[cfg(test)]
mod block_blobs_max;
#[cfg(test)]
mod block_bool;
#[cfg(test)]
mod block_comb;
#[cfg(test)]
mod block_enums;
#[cfg(test)]
mod block_f32;
#[cfg(test)]
mod block_f64;
#[cfg(test)]
mod block_i128;
#[cfg(test)]
mod block_i16;
#[cfg(test)]
mod block_i32;
#[cfg(test)]
mod block_i64;
#[cfg(test)]
mod block_i8;
#[cfg(test)]
mod block_u128;
#[cfg(test)]
mod block_u16;
#[cfg(test)]
mod block_u32;
#[cfg(test)]
mod block_u64;
#[cfg(test)]
mod block_u8;

#[cfg(test)]
pub(crate) use block_blob::*;
#[cfg(test)]
pub(crate) use block_blobs_max::*;
#[cfg(test)]
pub(crate) use block_bool::*;
#[cfg(test)]
pub(crate) use block_comb::*;
#[cfg(test)]
pub(crate) use block_enums::*;
#[cfg(test)]
pub(crate) use block_f32::*;
#[cfg(test)]
pub(crate) use block_f64::*;
#[cfg(test)]
pub(crate) use block_i128::*;
#[cfg(test)]
pub(crate) use block_i16::*;
#[cfg(test)]
pub(crate) use block_i32::*;
#[cfg(test)]
pub(crate) use block_i64::*;
#[cfg(test)]
pub(crate) use block_i8::*;
#[cfg(test)]
pub(crate) use block_u128::*;
#[cfg(test)]
pub(crate) use block_u16::*;
#[cfg(test)]
pub(crate) use block_u32::*;
#[cfg(test)]
pub(crate) use block_u64::*;
#[cfg(test)]
pub(crate) use block_u8::*;

use crate::*;
use proptest::prelude::*;

impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            BlockU8::arbitrary().prop_map(Block::BlockU8),
            BlockU16::arbitrary().prop_map(Block::BlockU16),
            BlockU32::arbitrary().prop_map(Block::BlockU32),
            BlockU64::arbitrary().prop_map(Block::BlockU64),
            // BlockU128::arbitrary().prop_map(Block::BlockU128),
            // BlockI8::arbitrary().prop_map(Block::BlockI8),
            // BlockI16::arbitrary().prop_map(Block::BlockI16),
            // BlockI32::arbitrary().prop_map(Block::BlockI32),
            // BlockI64::arbitrary().prop_map(Block::BlockI64),
            // BlockI128::arbitrary().prop_map(Block::BlockI128),
            // BlockF32::arbitrary().prop_map(Block::BlockF32),
            // BlockF64::arbitrary().prop_map(Block::BlockF64),
            // BlockBool::arbitrary().prop_map(Block::BlockBool),
            // BlockBlob::arbitrary().prop_map(Block::BlockBlob),
            // BlockBlobs::arbitrary().prop_map(Block::BlockBlobs),
            // BlockEnums::arbitrary().prop_map(Block::BlockEnums),
            // BlockCombination::arbitrary().prop_map(Block::BlockCombination),
        ]
        .boxed()
    }
}
