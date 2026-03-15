use brec::prelude::*;
use proptest::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct PayloadB {
    pub field_u8: Option<u8>,
    pub field_u16: Option<u16>,
    pub field_u32: Option<u32>,
    pub field_u64: Option<u64>,
    pub field_u128: Option<u128>,
    pub field_i8: Option<i8>,
    pub field_i16: Option<i16>,
    pub field_i32: Option<i32>,
    pub field_i64: Option<i64>,
    pub field_i128: Option<i128>,
    pub field_f32: Option<f32>,
    pub field_f64: Option<f64>,
    pub field_bool: Option<bool>,
    pub field_str: Option<String>,
    pub vec_u8: Option<Vec<u8>>,
    pub vec_u16: Option<Vec<u16>>,
    pub vec_u32: Option<Vec<u32>>,
    pub vec_u64: Option<Vec<u64>>,
    pub vec_u128: Option<Vec<u128>>,
    pub vec_i8: Option<Vec<i8>>,
    pub vec_i16: Option<Vec<i16>>,
    pub vec_i32: Option<Vec<i32>>,
    pub vec_i64: Option<Vec<i64>>,
    pub vec_i128: Option<Vec<i128>>,
    pub vec_str: Option<Vec<String>>,
}

impl Arbitrary for PayloadB {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            (
                prop::option::of(prop::collection::vec(any::<u8>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<u16>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<u32>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<u64>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<u128>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<i8>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<i16>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<i32>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<i64>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<i128>(), 0..100)).boxed(),
                prop::option::of(prop::collection::vec(any::<String>(), 0..50)).boxed(),
            )
                .boxed(),
            (
                prop::option::of(proptest::num::u8::ANY),
                prop::option::of(proptest::num::u16::ANY),
                prop::option::of(proptest::num::u32::ANY),
                prop::option::of(proptest::num::u64::ANY),
                prop::option::of(proptest::num::u128::ANY),
                prop::option::of(proptest::num::i8::ANY),
                prop::option::of(proptest::num::i16::ANY),
                prop::option::of(proptest::num::i32::ANY),
                prop::option::of(proptest::num::i64::ANY),
                prop::option::of(proptest::num::i128::ANY),
                prop::option::of(
                    proptest::num::f32::ANY.prop_filter("no NaNs or infinite", |f| f.is_finite()),
                ),
                prop::option::of(proptest::bool::ANY),
            )
                .boxed(),
            prop::option::of(
                proptest::num::f64::ANY.prop_filter("no NaNs or infinite", |f| f.is_finite()),
            )
            .boxed(),
            prop::option::of(any::<String>()),
        )
            .prop_map(
                move |(
                    (
                        vec_u8,
                        vec_u16,
                        vec_u32,
                        vec_u64,
                        vec_u128,
                        vec_i8,
                        vec_i16,
                        vec_i32,
                        vec_i64,
                        vec_i128,
                        vec_str,
                    ),
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
                    field_str,
                )| {
                    PayloadB {
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
                        field_str,
                        vec_u8,
                        vec_u16,
                        vec_u32,
                        vec_u64,
                        vec_u128,
                        vec_i8,
                        vec_i16,
                        vec_i32,
                        vec_i64,
                        vec_i128,
                        vec_str,
                    }
                },
            )
            .boxed()
    }
}
