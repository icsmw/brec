use crate::*;
use std::fmt;

pub struct ApiBlock;
pub struct ApiPayload;
pub struct ApiPacket;

pub trait Api: FormattableTs + FormattableRust {}

impl<T: FormattableTs + FormattableRust> Api for T {}

pub struct ApiModule<'a> {
    pub apis: Vec<&'a dyn Api>,
    pub mods: Vec<&'a dyn ImportableExportable>,
}

impl<'a> ApiModule<'a> {
    pub fn new(apis: Vec<&'a dyn Api>, mods: Vec<&'a dyn ImportableExportable>) -> Self {
        Self { apis, mods }
    }
}

pub trait FormattableTs {
    fn write_ts(&self, writer: &mut FormatterWriter) -> fmt::Result;
}

pub trait FormattableRust {
    fn write_rust(&self, writer: &mut FormatterWriter) -> fmt::Result;
}

pub trait ApiMethods {
    const ENCODE_METHOD_NAME: &'static str;
    const DECODE_METHOD_NAME: &'static str;
    fn snake_case_encode_method_name() -> &'static str {
        Self::ENCODE_METHOD_NAME
    }
    fn snake_case_decode_method_name() -> &'static str {
        Self::DECODE_METHOD_NAME
    }
    fn camel_case_encode_method_name() -> String {
        to_lower_camel_case(Self::ENCODE_METHOD_NAME)
    }
    fn camel_case_decode_method_name() -> String {
        to_lower_camel_case(Self::DECODE_METHOD_NAME)
    }
}

fn to_lower_camel_case(value: &str) -> String {
    let mut parts = value.split('_');
    let Some(first) = parts.next() else {
        return String::new();
    };
    let mut out = first.to_owned();
    for part in parts {
        let mut chars = part.chars();
        match chars.next() {
            Some(first) => {
                out.push_str(&first.to_uppercase().collect::<String>());
                out.push_str(chars.as_str());
            }
            None => {}
        }
    }
    out
}

impl ApiMethods for ApiBlock {
    const ENCODE_METHOD_NAME: &'static str = "encode_block";
    const DECODE_METHOD_NAME: &'static str = "decode_block";
}

impl ApiMethods for ApiPayload {
    const ENCODE_METHOD_NAME: &'static str = "encode_payload";
    const DECODE_METHOD_NAME: &'static str = "decode_payload";
}

impl ApiMethods for ApiPacket {
    const ENCODE_METHOD_NAME: &'static str = "encode_packet";
    const DECODE_METHOD_NAME: &'static str = "decode_packet";
}
