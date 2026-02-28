use brec::prelude::*;
use proptest::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum PayloadD {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    String(String),
}

impl Arbitrary for PayloadD {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            proptest::num::u8::ANY.prop_map(PayloadD::U8),
            proptest::num::u16::ANY.prop_map(PayloadD::U16),
            proptest::num::u32::ANY.prop_map(PayloadD::U32),
            proptest::num::u64::ANY.prop_map(PayloadD::U64),
            proptest::num::u128::ANY.prop_map(PayloadD::U128),
            proptest::num::i8::ANY.prop_map(PayloadD::I8),
            proptest::num::i16::ANY.prop_map(PayloadD::I16),
            proptest::num::i32::ANY.prop_map(PayloadD::I32),
            proptest::num::i64::ANY.prop_map(PayloadD::I64),
            proptest::num::i128::ANY.prop_map(PayloadD::I128),
            any::<String>().prop_map(PayloadD::String)
        ]
        .boxed()
    }
}
