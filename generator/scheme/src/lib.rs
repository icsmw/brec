use serde::{Deserialize, Serialize};

pub use brec_macros_parser::{BlockField, BlockTy, PayloadField, PayloadTy, Vis};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeFile {
    pub version: String,
    pub package: String,
    pub config: SchemeConfig,
    pub blocks: Vec<SchemeBlock>,
    pub payloads: Vec<SchemePayload>,
    #[serde(default)]
    pub types: Vec<SchemeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeConfig {
    pub no_default_payloads: bool,
    pub default_payloads: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeBlock {
    pub name: String,
    pub fullname: String,
    pub fullpath: String,
    pub visibility: Vis,
    pub no_crc: bool,
    pub fields: Vec<SchemeBlockField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemePayload {
    pub name: String,
    pub fullname: String,
    pub fullpath: String,
    pub is_ctx: bool,
    pub is_bincode: bool,
    pub is_crypt: bool,
    pub no_crc: bool,
    pub no_auto_crc: bool,
    pub no_default_sig: bool,
    pub hooks: bool,
    pub fields: Vec<SchemePayloadField>,
    pub variants: Vec<SchemePayloadVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeType {
    pub name: String,
    pub fullname: String,
    pub fullpath: String,
    pub fields: Vec<SchemePayloadField>,
    pub variants: Vec<SchemePayloadVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeBlockField {
    pub name: String,
    pub visibility: Vis,
    pub ty: SchemeFieldType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemePayloadField {
    pub name: Option<String>,
    pub visibility: Option<Vis>,
    pub ty: SchemeFieldType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemePayloadVariant {
    pub name: String,
    pub fields: Vec<SchemePayloadField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemeFieldType {
    Block(BlockTy),
    Payload(PayloadTy),
}
