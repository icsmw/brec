use brec::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum PayloadD {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    String(String),
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for PayloadD {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            proptest::num::u8::ANY.prop_map(PayloadD::U8),
            proptest::num::u16::ANY.prop_map(PayloadD::U16),
            proptest::num::u32::ANY.prop_map(PayloadD::U32),
            proptest::num::i8::ANY.prop_map(PayloadD::I8),
            proptest::num::i16::ANY.prop_map(PayloadD::I16),
            proptest::num::i32::ANY.prop_map(PayloadD::I32),
            any::<String>().prop_map(PayloadD::String)
        ]
        .boxed()
    }
}
