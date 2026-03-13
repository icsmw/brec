use prost::Message;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbMetadata {
    #[prost(uint32, tag = "1")]
    pub(crate) level: u32,
    #[prost(uint32, tag = "2")]
    pub(crate) target: u32,
    #[prost(uint64, tag = "3")]
    pub(crate) tm: u64,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbAttachment {
    #[prost(string, tag = "1")]
    pub(crate) uuid: String,
    #[prost(string, tag = "2")]
    pub(crate) name: String,
    #[prost(uint32, tag = "3")]
    pub(crate) chunk: u32,
    #[prost(bytes = "vec", tag = "4")]
    pub(crate) data: Vec<u8>,
    #[prost(map = "string, string", tag = "5")]
    pub(crate) fields: HashMap<String, String>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbRecord {
    #[prost(message, optional, tag = "1")]
    pub(crate) meta: Option<PbMetadata>,
    #[prost(string, tag = "2")]
    pub(crate) msg: String,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbRecordBinary {
    #[prost(message, optional, tag = "1")]
    pub(crate) meta: Option<PbMetadata>,
    #[prost(message, optional, tag = "2")]
    pub(crate) payload: Option<PbAttachment>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbRecordCrypt {
    #[prost(message, optional, tag = "1")]
    pub(crate) meta: Option<PbMetadata>,
    #[prost(bytes = "vec", tag = "2")]
    pub(crate) payload_encrypted: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbBlockBorrowed {
    #[prost(uint32, tag = "1")]
    pub(crate) field_u8: u32,
    #[prost(uint32, tag = "2")]
    pub(crate) field_u16: u32,
    #[prost(uint32, tag = "3")]
    pub(crate) field_u32: u32,
    #[prost(uint64, tag = "4")]
    pub(crate) field_u64: u64,
    #[prost(bytes = "vec", tag = "5")]
    pub(crate) field_u128: Vec<u8>,
    #[prost(int32, tag = "6")]
    pub(crate) field_i8: i32,
    #[prost(int32, tag = "7")]
    pub(crate) field_i16: i32,
    #[prost(int32, tag = "8")]
    pub(crate) field_i32: i32,
    #[prost(int64, tag = "9")]
    pub(crate) field_i64: i64,
    #[prost(bytes = "vec", tag = "10")]
    pub(crate) field_i128: Vec<u8>,
    #[prost(float, tag = "11")]
    pub(crate) field_f32: f32,
    #[prost(double, tag = "12")]
    pub(crate) field_f64: f64,
    #[prost(bool, tag = "13")]
    pub(crate) field_bool: bool,
    #[prost(bytes = "vec", tag = "14")]
    pub(crate) blob_a: Vec<u8>,
    #[prost(bytes = "vec", tag = "15")]
    pub(crate) blob_b: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PbRecordBorrowed {
    #[prost(message, optional, tag = "1")]
    pub(crate) block: Option<PbBlockBorrowed>,
}

pub(crate) enum PbPacket {
    Record(PbRecord),
    RecordBinary(PbRecordBinary),
    RecordCrypt(PbRecordCrypt),
    Borrowed(PbRecordBorrowed),
}

pub(crate) fn invalid_data<E: std::error::Error + Send + Sync + 'static>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
}
