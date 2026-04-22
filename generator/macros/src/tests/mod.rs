mod field;
mod packet;
mod r#struct;
mod tcrate;
mod value;

pub(crate) use field::*;
pub(crate) use packet::*;
pub(crate) use r#struct::*;
pub(crate) use tcrate::*;
pub(crate) use value::*;

use proc_macro2::TokenStream;
use proptest::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ENTITIES_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn gen_name(first_upper: bool) -> String {
    TEST_ENTITIES_NAME_INDEX.fetch_add(1, Ordering::Relaxed);
    format!(
        "{}ame{}",
        if first_upper { "N" } else { "n" },
        TEST_ENTITIES_NAME_INDEX.load(Ordering::Relaxed)
    )
}

#[derive(Debug, Default)]
pub(crate) enum Target {
    #[default]
    Block,
    Payload,
}

impl Target {
    pub fn primitive_values() -> BoxedStrategy<ValueId> {
        prop_oneof![
            Just(ValueId::U8),
            Just(ValueId::U16),
            Just(ValueId::U32),
            Just(ValueId::U64),
            Just(ValueId::U128),
            Just(ValueId::I8),
            Just(ValueId::I16),
            Just(ValueId::I32),
            Just(ValueId::I64),
            Just(ValueId::I128),
            Just(ValueId::F32),
            Just(ValueId::F64),
            Just(ValueId::Bool),
            Just(ValueId::String),
        ]
        .boxed()
    }
    pub fn block_values() -> BoxedStrategy<ValueId> {
        prop_oneof![
            Just(ValueId::U8),
            Just(ValueId::U16),
            Just(ValueId::U32),
            Just(ValueId::U64),
            Just(ValueId::U128),
            Just(ValueId::I8),
            Just(ValueId::I16),
            Just(ValueId::I32),
            Just(ValueId::I64),
            Just(ValueId::I128),
            Just(ValueId::F32),
            Just(ValueId::F64),
            Just(ValueId::Bool),
            Just(ValueId::Blob)
        ]
        .boxed()
    }

    pub fn nested_values() -> BoxedStrategy<ValueId> {
        prop_oneof![
            Just(ValueId::U8),
            Just(ValueId::U16),
            Just(ValueId::U32),
            Just(ValueId::U64),
            Just(ValueId::U128),
            Just(ValueId::I8),
            Just(ValueId::I16),
            Just(ValueId::I32),
            Just(ValueId::I64),
            Just(ValueId::I128),
            Just(ValueId::F32),
            Just(ValueId::F64),
            Just(ValueId::Bool),
            Just(ValueId::String),
            Just(ValueId::Option),
            Just(ValueId::Tuple),
            Just(ValueId::HashMap),
            Just(ValueId::Vec)
        ]
        .boxed()
    }

    pub fn payload_values() -> BoxedStrategy<ValueId> {
        prop_oneof![
            Just(ValueId::U8),
            Just(ValueId::U16),
            Just(ValueId::U32),
            Just(ValueId::U64),
            Just(ValueId::U128),
            Just(ValueId::I8),
            Just(ValueId::I16),
            Just(ValueId::I32),
            Just(ValueId::I64),
            Just(ValueId::I128),
            Just(ValueId::F32),
            Just(ValueId::F64),
            Just(ValueId::Bool),
            Just(ValueId::String),
            Just(ValueId::Option),
            Just(ValueId::Tuple),
            Just(ValueId::HashMap),
            Just(ValueId::Vec)
        ]
        .boxed()
    }
}

pub(crate) trait Generate {
    type Options;
    fn declaration(&self, opt: Self::Options) -> TokenStream;
    fn instance(&self, opt: Self::Options) -> TokenStream;
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(1)
    })]


    #[test]
    fn generate(tcrates in proptest::collection::vec(TCrate::arbitrary(), 10)) {
        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
        let tests_path = root.join("../gen_tests");
        if !tests_path.exists() {
            std::fs::create_dir(&tests_path)?;
        }
        for tcrate in tcrates.iter() {
            tcrate.write_all(&tests_path)?;
            println!("Created test-case with: {} packets", tcrate.packets.len());
        }
    }

}
