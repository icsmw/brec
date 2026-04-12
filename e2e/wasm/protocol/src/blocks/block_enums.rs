use brec::prelude::*;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

impl TryFrom<u8> for Level {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Level::Err),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Debug),
            3 => Ok(Level::Info),
            invalid => Err(format!("{invalid} isn't valid value for Level")),
        }
    }
}

impl From<&Level> for u8 {
    fn from(value: &Level) -> Self {
        match value {
            Level::Err => 0,
            Level::Warn => 1,
            Level::Debug => 2,
            Level::Info => 3,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    File,
    Stream,
    Socket,
}

impl TryFrom<u8> for Kind {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Kind::File),
            1 => Ok(Kind::Stream),
            2 => Ok(Kind::Socket),
            invalid => Err(format!("{invalid} isn't valid value for Kind")),
        }
    }
}

impl From<&Kind> for u8 {
    fn from(value: &Kind) -> Self {
        match value {
            Kind::File => 0,
            Kind::Stream => 1,
            Kind::Socket => 2,
        }
    }
}

#[block]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BlockEnums {
    pub level: Level,
    pub kind: Kind,
}

#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for Level {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            Just(Level::Err),
            Just(Level::Warn),
            Just(Level::Debug),
            Just(Level::Info)
        ]
        .boxed()
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for Kind {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![Just(Kind::File), Just(Kind::Stream), Just(Kind::Socket)].boxed()
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for BlockEnums {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (Level::arbitrary(), Kind::arbitrary())
            .prop_map(|(level, kind)| BlockEnums { level, kind })
            .boxed()
    }
}
