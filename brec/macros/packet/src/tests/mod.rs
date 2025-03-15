mod r#enum;
mod field;
mod packet;
mod r#struct;
mod tcrate;
mod value;

pub(crate) use field::*;
pub(crate) use packet::*;
// pub(crate) use r#enum::*;
pub(crate) use r#struct::*;
pub(crate) use tcrate::*;
pub(crate) use value::*;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use std::{collections::HashSet, sync::Mutex};

lazy_static! {
    pub(crate) static ref TEST_ENTITIES_NAMES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub(crate) fn chk_name<S: AsRef<str>>(name: S) -> bool {
    let mut names = TEST_ENTITIES_NAMES.lock().unwrap();
    if names.contains(name.as_ref()) {
        false
    } else {
        names.insert(name.as_ref().to_owned());
        true
    }
}

#[derive(Debug, Default)]
pub(crate) enum Target {
    #[default]
    Block,
    Payload,
}

impl Target {
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
            // Just(ValueId::Blob),
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
    fn generate(tcrates in proptest::collection::vec(TCrate::arbitrary(), 1..2)) {
        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
        let tests_path = root.join("../../../gen_tests");
        if !tests_path.exists() {
            std::fs::create_dir(&tests_path)?;
        }
        for tcrate in tcrates.iter() {
            tcrate.write_all(&tests_path)?;
        }
    }

}
