use brec::prelude::*;
use proptest::prelude::*;

#[block]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockCombination {
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    blob_a: [u8; 1],
    blob_b: [u8; 100],
    blob_c: [u8; 1000],
    blob_d: [u8; 10000],
}

impl Arbitrary for BlockCombination {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            (
                prop::collection::vec(any::<u8>(), 1).boxed(),
                prop::collection::vec(any::<u8>(), 100).boxed(),
                prop::collection::vec(any::<u8>(), 1000).boxed(),
                prop::collection::vec(any::<u8>(), 10000).boxed(),
            )
                .boxed(),
            (
                proptest::num::u8::ANY,
                proptest::num::u16::ANY,
                proptest::num::u32::ANY,
                proptest::num::u64::ANY,
                proptest::num::u128::ANY,
                proptest::num::i8::ANY,
                proptest::num::i16::ANY,
                proptest::num::i32::ANY,
                proptest::num::i64::ANY,
                proptest::num::i128::ANY,
                proptest::num::f32::ANY,
                proptest::bool::ANY,
            )
                .boxed(),
            proptest::num::f64::ANY.boxed(),
        )
            .prop_map(
                move |(
                    (blob_a_vec, blob_b_vec, blob_c_vec, blob_d_vec),
                    (
                        field_u8,
                        field_u16,
                        field_u32,
                        field_u64,
                        field_u128,
                        field_i8,
                        field_i16,
                        field_i32,
                        field_i64,
                        field_i128,
                        field_f32,
                        field_bool,
                    ),
                    field_f64,
                )| {
                    let mut blob_a = [0u8; 1];
                    let mut blob_b = [0u8; 100];
                    let mut blob_c = [0u8; 1000];
                    let mut blob_d = [0u8; 10000];

                    blob_a.copy_from_slice(&blob_a_vec);
                    blob_b.copy_from_slice(&blob_b_vec);
                    blob_c.copy_from_slice(&blob_c_vec);
                    blob_d.copy_from_slice(&blob_d_vec);
                    BlockCombination {
                        field_u8,
                        field_u16,
                        field_u32,
                        field_u64,
                        field_u128,
                        field_i8,
                        field_i16,
                        field_i32,
                        field_i64,
                        field_i128,
                        field_f32,
                        field_f64,
                        field_bool,
                        blob_a,
                        blob_b,
                        blob_c,
                        blob_d,
                    }
                },
            )
            .boxed()
    }
}
