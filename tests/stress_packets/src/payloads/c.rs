use brec::prelude::*;
use proptest::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCA {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
}

impl Arbitrary for NestedStructCA {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            proptest::num::u8::ANY,
            proptest::num::u16::ANY,
            proptest::num::u32::ANY,
        )
            .prop_map(|(field_u8, field_u16, field_u32)| NestedStructCA {
                field_u8,
                field_u16,
                field_u32,
            })
            .boxed()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCB {
    pub field_i8: i8,
    pub field_i16: i16,
    pub field_i32: i32,
}

impl Arbitrary for NestedStructCB {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            proptest::num::i8::ANY,
            proptest::num::i16::ANY,
            proptest::num::i32::ANY,
        )
            .prop_map(|(field_i8, field_i16, field_i32)| NestedStructCB {
                field_i8,
                field_i16,
                field_i32,
            })
            .boxed()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct NestedStructCC {
    pub field_bool: bool,
    pub field_str: String,
    pub vec_u8: Vec<u8>,
}

impl Arbitrary for NestedStructCC {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            proptest::bool::ANY.boxed(),
            any::<String>().boxed(),
            prop::collection::vec(any::<u8>(), 0..100).boxed(),
        )
            .prop_map(|(field_bool, field_str, vec_u8)| NestedStructCC {
                field_bool,
                field_str,
                vec_u8,
            })
            .boxed()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum NestedEnumC {
    One(String),
    Two(Vec<u8>),
    Three,
    Four(NestedStructCC),
}

impl Arbitrary for NestedEnumC {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            any::<String>().prop_map(NestedEnumC::One),
            prop::collection::vec(any::<u8>(), 0..100).prop_map(NestedEnumC::Two),
            Just(NestedEnumC::Three),
            NestedStructCC::arbitrary().prop_map(NestedEnumC::Four)
        ]
        .boxed()
    }
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct PayloadC {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_struct_a: NestedStructCA,
    pub field_struct_b: Option<NestedStructCB>,
    pub field_struct_c: Vec<NestedStructCC>,
    pub field_enum: NestedEnumC,
    pub vec_enum: Vec<NestedEnumC>,
}

impl Arbitrary for PayloadC {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            proptest::num::u8::ANY,
            proptest::num::u16::ANY,
            proptest::num::u32::ANY,
            proptest::num::u64::ANY,
            proptest::num::u128::ANY,
            NestedStructCA::arbitrary(),
            prop::option::of(NestedStructCB::arbitrary()),
            prop::collection::vec(any::<NestedStructCC>(), 0..100),
            NestedEnumC::arbitrary(),
            prop::collection::vec(any::<NestedEnumC>(), 0..100),
        )
            .prop_map(
                move |(
                    field_u8,
                    field_u16,
                    field_u32,
                    field_u64,
                    field_u128,
                    field_struct_a,
                    field_struct_b,
                    field_struct_c,
                    field_enum,
                    vec_enum,
                )| {
                    PayloadC {
                        field_u8,
                        field_u16,
                        field_u32,
                        field_u64,
                        field_u128,
                        field_struct_a,
                        field_struct_b,
                        field_struct_c,
                        field_enum,
                        vec_enum,
                    }
                },
            )
            .boxed()
    }
}
