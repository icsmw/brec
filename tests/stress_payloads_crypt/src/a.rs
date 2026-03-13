use brec::prelude::*;
use proptest::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug)]
pub struct PayloadA {
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
    pub field_str: String,
    pub vec_u8: Vec<u8>,
    pub vec_u16: Vec<u16>,
    pub vec_u32: Vec<u32>,
    pub vec_u64: Vec<u64>,
    pub vec_u128: Vec<u128>,
    pub vec_i8: Vec<i8>,
    pub vec_i16: Vec<i16>,
    pub vec_i32: Vec<i32>,
    pub vec_i64: Vec<i64>,
    pub vec_i128: Vec<i128>,
    pub vec_str: Vec<String>,
}

impl Arbitrary for PayloadA {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            (
                prop::collection::vec(any::<u8>(), 0..100).boxed(),
                prop::collection::vec(any::<u16>(), 0..100).boxed(),
                prop::collection::vec(any::<u32>(), 0..100).boxed(),
                prop::collection::vec(any::<u64>(), 0..100).boxed(),
                prop::collection::vec(any::<u128>(), 0..100).boxed(),
                prop::collection::vec(any::<i8>(), 0..100).boxed(),
                prop::collection::vec(any::<i16>(), 0..100).boxed(),
                prop::collection::vec(any::<i32>(), 0..100).boxed(),
                prop::collection::vec(any::<i64>(), 0..100).boxed(),
                prop::collection::vec(any::<i128>(), 0..100).boxed(),
                prop::collection::vec(any::<String>(), 0..50).boxed(),
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
                proptest::num::f32::ANY.prop_filter("no NaNs or infinite", |f| f.is_finite()),
                proptest::bool::ANY,
            )
                .boxed(),
            proptest::num::f64::ANY
                .prop_filter("no NaNs or infinite", |f| f.is_finite())
                .boxed(),
            any::<String>(),
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
                    PayloadA {
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
