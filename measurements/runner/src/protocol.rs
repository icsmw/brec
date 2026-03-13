use brec::prelude::*;

use crate::{
    content::MatchValue,
    report::PayloadKind,
    test::{Block, Payload, WrappedPacket, MATCH},
    JSONRow, TextualRow,
};
use proptest::prelude::*;
use std::{collections::BTreeMap, fmt};

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
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct BlockBorrowed {
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
    pub blob_a: [u8; 100],
    pub blob_b: [u8; 255],
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RecordBincode {
    pub mt: Metadata,
    pub atc: AttachmentBincode,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RecordCrypt {
    pub mt: Metadata,
    pub atc: AttachmentCrypt,
}

pub trait MeasurementRecord:
    Arbitrary<Parameters = (), Strategy = BoxedStrategy<Self>> + Clone + fmt::Debug + Sized
{
    type JsonPayload: MatchValue
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Clone
        + fmt::Debug
        + PartialEq
        + PartialOrd;

    const PAYLOAD: PayloadKind;

    fn metadata(&self) -> &Metadata;
    fn text_value(&self) -> String;
    fn json_payload(&self) -> Self::JsonPayload;
    fn packet_payload(&self) -> Payload;

    fn into_text_row(&self) -> TextualRow {
        let meta = self.metadata().clone();
        TextualRow {
            msg: format!(
                "{}{} {} {}",
                meta.level,
                meta.target,
                meta.tm,
                self.text_value()
            ),
        }
    }

    fn into_json_row(&self) -> JSONRow<Self::JsonPayload> {
        JSONRow {
            meta: self.metadata().clone(),
            payload: self.json_payload(),
        }
    }

    fn into_packet(&self) -> WrappedPacket {
        WrappedPacket {
            blocks: vec![Block::Metadata(self.metadata().clone())],
            payload: Some(self.packet_payload()),
        }
    }
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

impl Arbitrary for BlockBorrowed {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            (
                any::<u8>(),
                any::<u16>(),
                any::<u32>(),
                any::<u64>(),
                any::<u128>(),
                any::<i8>(),
                any::<i16>(),
                any::<i32>(),
                any::<i64>(),
                any::<i128>(),
                any::<f32>(),
                any::<f64>(),
            ),
            (
                any::<bool>(),
                proptest::collection::vec(any::<u8>(), 100),
                proptest::collection::vec(any::<u8>(), 255),
            ),
        )
            .prop_map(
                |(
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
                        field_f64,
                    ),
                    (field_bool, blob_a, blob_b),
                )| BlockBorrowed {
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
                    blob_a: blob_a.try_into().expect("blob_a has fixed size"),
                    blob_b: blob_b.try_into().expect("blob_b has fixed size"),
                },
            )
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

impl Arbitrary for RecordBincode {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (any::<Metadata>(), any::<AttachmentBincode>(), 0..100usize)
            .prop_map(|(mt, mut atc, rate)| {
                if rate > 50 {
                    atc.name = format!("{}{MATCH}{}", atc.name, atc.chunk);
                }
                RecordBincode { mt, atc }
            })
            .boxed()
    }
}

impl Arbitrary for RecordCrypt {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (any::<Metadata>(), any::<AttachmentCrypt>(), 0..100usize)
            .prop_map(|(mt, mut atc, rate)| {
                if rate > 50 {
                    atc.name = format!("{}{MATCH}{}", atc.name, atc.chunk);
                }
                RecordCrypt { mt, atc }
            })
            .boxed()
    }
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct AttachmentBincode {
    pub uuid: String,
    pub name: String,
    pub chunk: u32,
    pub data: Vec<u8>,
    pub fields: BTreeMap<String, String>,
}

impl Arbitrary for AttachmentBincode {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            "[a-f0-9]{32}",
            ".{1,32}",
            any::<u32>(),
            proptest::collection::vec(any::<u8>(), 10..250),
            proptest::collection::btree_map(".{1,20}", ".{1,250}", 0..10),
        )
            .prop_map(|(uuid, name, chunk, data, fields)| Self {
                uuid,
                name,
                chunk,
                data,
                fields,
            })
            .boxed()
    }
}

#[payload(bincode, crypt)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct AttachmentCrypt {
    pub uuid: String,
    pub name: String,
    pub chunk: u32,
    pub data: Vec<u8>,
    pub fields: BTreeMap<String, String>,
}

impl Arbitrary for AttachmentCrypt {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            "[a-f0-9]{32}",
            ".{1,32}",
            any::<u32>(),
            proptest::collection::vec(any::<u8>(), 10..250),
            proptest::collection::btree_map(".{1,20}", ".{1,250}", 0..10),
        )
            .prop_map(|(uuid, name, chunk, data, fields)| Self {
                uuid,
                name,
                chunk,
                data,
                fields,
            })
            .boxed()
    }
}

impl MeasurementRecord for Record {
    type JsonPayload = String;

    const PAYLOAD: PayloadKind = PayloadKind::Record;

    fn metadata(&self) -> &Metadata {
        &self.mt
    }

    fn text_value(&self) -> String {
        self.msg.clone()
    }

    fn json_payload(&self) -> Self::JsonPayload {
        self.msg.clone()
    }

    fn packet_payload(&self) -> Payload {
        Payload::String(self.msg.clone())
    }
}

impl MeasurementRecord for RecordBincode {
    type JsonPayload = AttachmentBincode;

    const PAYLOAD: PayloadKind = PayloadKind::RecordBincode;

    fn metadata(&self) -> &Metadata {
        &self.mt
    }

    fn text_value(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.atc.uuid,
            self.atc.name,
            self.atc.chunk,
            serde_json::to_string(&self.atc.data).expect("Serialize attachment data"),
            serde_json::to_string(&self.atc.fields).expect("Serialize attachment data")
        )
    }

    fn json_payload(&self) -> Self::JsonPayload {
        self.atc.clone()
    }

    fn packet_payload(&self) -> Payload {
        Payload::AttachmentBincode(self.atc.clone())
    }
}

impl MeasurementRecord for RecordCrypt {
    type JsonPayload = AttachmentCrypt;

    const PAYLOAD: PayloadKind = PayloadKind::RecordCrypt;

    fn metadata(&self) -> &Metadata {
        &self.mt
    }

    fn text_value(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.atc.uuid,
            self.atc.name,
            self.atc.chunk,
            serde_json::to_string(&self.atc.data).expect("Serialize attachment data"),
            serde_json::to_string(&self.atc.fields).expect("Serialize attachment data")
        )
    }

    fn json_payload(&self) -> Self::JsonPayload {
        self.atc.clone()
    }

    fn packet_payload(&self) -> Payload {
        Payload::AttachmentCrypt(self.atc.clone())
    }
}
