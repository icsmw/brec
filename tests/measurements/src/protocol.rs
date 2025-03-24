use brec::prelude::*;

use crate::test::MATCH;
use proptest::prelude::*;
use std::fmt;

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::Err => "[ERR]",
                Level::Warn => "[WARN]",
                Level::Debug => "[DEBUG]",
                Level::Info => "[INFO]",
            }
        )
    }
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

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
pub enum Target {
    Server,
    Client,
    Proxy,
}

impl TryFrom<u8> for Target {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Target::Server),
            1 => Ok(Target::Client),
            2 => Ok(Target::Proxy),
            invalid => Err(format!("{invalid} isn't valid value for Target")),
        }
    }
}

impl From<&Target> for u8 {
    fn from(value: &Target) -> Self {
        match value {
            Target::Server => 0,
            Target::Client => 1,
            Target::Proxy => 2,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Target::Server => "[Server]",
                Target::Client => "[Client]",
                Target::Proxy => "[Proxy]",
            }
        )
    }
}

#[block]
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
pub struct Metadata {
    pub level: Level,
    pub target: Target,
    pub tm: u64,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Record {
    pub mt: Metadata,
    pub msg: String,
}

impl Arbitrary for Level {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (0u8..=3u8)
            .prop_map(|level| level.try_into().expect("Valid Level"))
            .boxed()
    }
}

impl Arbitrary for Target {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (0u8..=2u8)
            .prop_map(|target| target.try_into().expect("Valid Target"))
            .boxed()
    }
}

impl Arbitrary for Metadata {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            any::<Level>(),
            any::<Target>(),
            (10_000_000u64..20_000_000u64),
        )
            .prop_map(|(level, target, tm)| Metadata { level, target, tm })
            .boxed()
    }
}

impl Arbitrary for Record {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (any::<Metadata>(), (0..100), "[a-zA-Z]{100,1000}")
            .prop_map(|(mt, rate, msg)| Record {
                mt,
                msg: if rate > 50 {
                    format!("{msg}{MATCH}{msg}")
                } else {
                    msg
                },
            })
            .boxed()
    }
}

// #[payload(bincode)]
// #[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone, Debug)]
// pub struct Attachment {
//     pub uuid: String,
//     pub name: String,
//     pub chunk: u32,
//     pub data: Vec<u8>,
// }
