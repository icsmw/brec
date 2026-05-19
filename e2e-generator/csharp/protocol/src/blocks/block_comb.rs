use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockCombination {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_i8: i8,
    pub field_i16: i16,
    pub field_i32: i32,
    pub field_i64: i64,
    pub field_i128: i128,
    pub field_f32: f32,
    pub field_f64: f64,
    pub field_bool: bool,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockCombination {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
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
                proptest::num::f32::ANY.prop_filter("no NaNs or infinite", |f| f.is_finite()),
                proptest::bool::ANY,
            )
                .boxed(),
            proptest::num::f64::ANY
                .prop_filter("no NaNs or infinite", |f| f.is_finite())
                .boxed(),
        )
            .prop_map(
                move |(
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
                    }
                },
            )
            .boxed()
    }
}
