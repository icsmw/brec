mod error;
mod packet;

use crate::Error;
pub use error::*;
pub use packet::*;
use wasm_bindgen::{JsCast, JsValue};

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Clone, Copy, Debug)]
pub enum WasmFieldHint {
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
pub fn from_value<T: JsCast>(hint: WasmFieldHint, value: JsValue) -> Result<T, Error> {
    value
        .dyn_into::<T>()
        .map_err(|_| WasmError::invalid_field(hint, "type conversion failed"))
}

#[inline]
pub fn from_value_name<T: JsCast>(name: impl Into<String>, value: JsValue) -> Result<T, Error> {
    value
        .dyn_into::<T>()
        .map_err(|_| WasmError::invalid_field_name(name, "type conversion failed"))
}
