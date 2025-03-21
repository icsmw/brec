use proptest::prelude::*;

use crate::{Level, Target};

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
                        format!("{msg}-match-{msg}")
                    } else {
                        msg
                    }
                ),
            })
            .boxed()
    }
}
