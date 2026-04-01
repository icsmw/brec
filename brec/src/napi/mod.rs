mod error;
mod packet;

use crate::Error;
pub use error::*;
use napi::{Unknown, bindgen_prelude::FromNapiValue};
pub use packet::*;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Clone, Copy, Debug)]
pub enum NapiFieldHint {
    Bool,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    I64,
    U64,
    I128,
    U128,
    String,
    F64,
    Vec,
    Option,
    Blob,
    Blocks,
    Payload,
    Object,
}

#[inline]
pub fn from_unknown<T: FromNapiValue>(hint: NapiFieldHint, value: Unknown<'_>) -> Result<T, Error> {
    FromNapiValue::from_unknown(value).map_err(|err| NapiError::invalid_field(hint, err))
}

#[inline]
pub fn from_unknown_name<T: FromNapiValue>(
    name: impl Into<String>,
    value: Unknown<'_>,
) -> Result<T, Error> {
    FromNapiValue::from_unknown(value).map_err(|err| NapiError::invalid_field_name(name, err))
}
