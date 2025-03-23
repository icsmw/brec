use proptest::prelude::*;

use crate::{test::MATCH, Level, Metadata, Target};

#[derive(Debug)]
pub struct TextualRow {
    pub msg: String,
}

impl Arbitrary for TextualRow {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            any::<Level>(),
            any::<Target>(),
            (10_000_000u64..20_000_000u64),
            (0..100),
            "[a-zA-Z]{100,1000}",
        )
            .prop_map(|(level, target, tm, rate, msg)| TextualRow {
                msg: format!(
                    "{level}{target} {tm} {}",
                    if rate > 50 {
                        format!("{msg}{MATCH}{msg}")
                    } else {
                        msg
                    }
                ),
            })
            .boxed()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
pub struct JSONRow {
    pub meta: Metadata,
    pub msg: String,
}

impl Arbitrary for JSONRow {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (any::<Metadata>(), (0..100), "[a-zA-Z]{100,1000}")
            .prop_map(|(meta, rate, msg)| JSONRow {
                msg: if rate > 50 {
                    format!("{msg}{MATCH}{msg}")
                } else {
                    msg
                },
                meta,
            })
            .boxed()
    }
}
