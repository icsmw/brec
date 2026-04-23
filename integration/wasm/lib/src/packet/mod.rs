#[macro_use]
mod macros;

use super::{WasmError, WasmFieldHint, WasmFieldHintId};
use brec_consts::*;
use js_sys::{Array, BigInt, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

/// Rust <-> JS object conversion contract used by `wasm` helpers.
pub trait WasmObject: Sized {
    /// Converts this value into a JavaScript object representation.
    fn to_wasm_object(&self) -> Result<JsValue, WasmError>;
    /// Constructs this value from a JavaScript object representation.
    fn from_wasm_object(value: JsValue) -> Result<Self, WasmError>;
}

/// Schema-driven Rust <-> JS conversion used by payload nested types.
pub trait WasmConvert: Sized {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError>;
    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError>;
}

fn get_object(value: JsValue, field: WasmFieldHint) -> Result<js_sys::Object, WasmError> {
    value
        .dyn_into::<js_sys::Object>()
        .map_err(|_| WasmError::invalid_field(field, "expected object"))
}

fn get_field(obj: &js_sys::Object, name: &str, field: WasmFieldHint) -> Result<JsValue, WasmError> {
    let key = JsValue::from_str(name);
    let has = Reflect::has(obj, &key)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to access field {name}")))?;
    if !has {
        return Err(WasmError::MissingField(name.to_owned()));
    }
    Reflect::get(obj, &key)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to read field {name}")))
}

fn set_field(
    obj: &js_sys::Object,
    name: &str,
    value: &JsValue,
    field: WasmFieldHint,
) -> Result<(), WasmError> {
    let key = JsValue::from_str(name);
    Reflect::set(obj, &key, value)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to write field {name}")))?;
    Ok(())
}

impl WasmConvert for bool {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        Ok(JsValue::from_bool(*self))
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        value
            .as_bool()
            .ok_or_else(|| WasmError::invalid_field(WasmFieldHint::Bool, "expected boolean"))
    }
}

impl WasmConvert for String {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        Ok(JsValue::from_str(self))
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        value
            .as_string()
            .ok_or_else(|| WasmError::invalid_field(WasmFieldHint::String, "expected string"))
    }
}

impl_wasm_simple!(
    u8 => WasmFieldHint::U8,
    u16 => WasmFieldHint::U16,
    u32 => WasmFieldHint::U32,
    i8 => WasmFieldHint::I8,
    i16 => WasmFieldHint::I16,
    i32 => WasmFieldHint::I32,
);

impl_wasm_bigint_signed!(
    i64 => (WasmFieldHint::I64, WasmFieldHintId::I64),
    i128 => (WasmFieldHint::I128, WasmFieldHintId::I128),
);

impl_wasm_bigint_unsigned!(
    u64 => (WasmFieldHint::U64, WasmFieldHintId::U64),
    u128 => (WasmFieldHint::U128, WasmFieldHintId::U128),
);

impl WasmConvert for f32 {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        <u32 as WasmConvert>::to_wasm_value(&self.to_bits())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        let bits = <u32 as WasmConvert>::from_wasm_value(value)?;
        Ok(f32::from_bits(bits))
    }
}

impl WasmConvert for f64 {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        Ok(BigInt::from(self.to_bits()).into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        let bits = <u64 as WasmConvert>::from_wasm_value(value).map_err(|_| {
            WasmError::invalid_field(WasmFieldHint::F64, "expected BigInt for f64 bits")
        })?;
        Ok(f64::from_bits(bits))
    }
}

impl<T: WasmConvert> WasmConvert for Vec<T> {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        let arr = Array::new_with_length(self.len() as u32);
        for (idx, item) in self.iter().enumerate() {
            arr.set(idx as u32, item.to_wasm_value()?);
        }
        Ok(arr.into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        if !Array::is_array(&value) {
            return Err(WasmError::invalid_field(
                WasmFieldHint::Vec,
                "expected array",
            ));
        }
        let arr = value
            .dyn_into::<Array>()
            .map_err(|_| WasmError::invalid_field(WasmFieldHint::Vec, "expected array"))?;
        let mut out = Vec::with_capacity(arr.length() as usize);
        for idx in 0..arr.length() {
            out.push(T::from_wasm_value(arr.get(idx))?);
        }
        Ok(out)
    }
}

impl<T: WasmConvert> WasmConvert for Option<T> {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        match self {
            Some(v) => v.to_wasm_value(),
            None => Ok(JsValue::NULL),
        }
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        if value.is_null() || value.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(T::from_wasm_value(value)?))
        }
    }
}

impl<const N: usize> WasmConvert for [u8; N] {
    fn to_wasm_value(&self) -> Result<JsValue, WasmError> {
        let view = Uint8Array::from(self.as_slice());
        Ok(view.into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, WasmError> {
        let raw = if value.is_instance_of::<Uint8Array>() {
            let arr = value.dyn_into::<Uint8Array>().map_err(|_| {
                WasmError::invalid_field(WasmFieldHint::Blob, "expected Uint8Array")
            })?;
            let mut out = vec![0u8; arr.length() as usize];
            arr.copy_to(&mut out);
            out
        } else {
            <Vec<u8> as WasmConvert>::from_wasm_value(value)?
        };
        raw.try_into().map_err(|bytes: Vec<u8>| {
            WasmError::InvalidField(
                WasmFieldHintId::Blob.to_string(),
                format!("expected {N} bytes, got {}", bytes.len()),
            )
        })
    }
}

/// Converts packet into `{ blocks: Array<{}>, payload: {} | null }`.
pub fn to_wasm_object<Block: WasmObject, Payload: WasmObject>(
    blocks: &[Block],
    payload: Option<&Payload>,
) -> Result<JsValue, WasmError> {
    let obj = js_sys::Object::new();
    let js_blocks = Array::new_with_length(blocks.len() as u32);
    for (idx, block) in blocks.iter().enumerate() {
        js_blocks.set(idx as u32, block.to_wasm_object()?);
    }
    set_field(
        &obj,
        BLOCKS_FIELD_NAME,
        &js_blocks.into(),
        WasmFieldHint::Blocks,
    )?;

    let payload = match payload {
        Some(payload) => payload.to_wasm_object()?,
        None => JsValue::NULL,
    };
    set_field(&obj, PAYLOAD_FIELD_NAME, &payload, WasmFieldHint::Payload)?;
    Ok(obj.into())
}

/// Parses packet from `{ blocks: Array<{}>, payload: {} | null | undefined }`.
pub fn from_wasm_object<Block: WasmObject, Payload: WasmObject>(
    value: JsValue,
) -> Result<(Vec<Block>, Option<Payload>), WasmError> {
    let obj = get_object(value, WasmFieldHint::Object)
        .map_err(|err| WasmError::InvalidObject(err.to_string()))?;

    let blocks_raw = get_field(&obj, BLOCKS_FIELD_NAME, WasmFieldHint::Blocks)?;
    if !Array::is_array(&blocks_raw) {
        return Err(WasmError::invalid_field(
            WasmFieldHint::Blocks,
            "expected array for blocks",
        ));
    }
    let blocks_arr = Array::from(&blocks_raw);
    let mut blocks = Vec::with_capacity(blocks_arr.length() as usize);
    for idx in 0..blocks_arr.length() {
        blocks.push(Block::from_wasm_object(blocks_arr.get(idx)).map_err(|err| {
            WasmError::InvalidField(
                WasmFieldHintId::Blocks.to_string(),
                format!("index {idx}: {err}"),
            )
        })?);
    }

    let payload_key = JsValue::from_str(PAYLOAD_FIELD_NAME);
    let payload = if Reflect::has(&obj, &payload_key).map_err(|_| {
        WasmError::invalid_field(WasmFieldHint::Payload, "failed to inspect payload")
    })? {
        let raw = Reflect::get(&obj, &payload_key).map_err(|_| {
            WasmError::invalid_field(WasmFieldHint::Payload, "failed to read payload")
        })?;
        if raw.is_null() || raw.is_undefined() {
            None
        } else {
            Some(Payload::from_wasm_object(raw)?)
        }
    } else {
        None
    };

    Ok((blocks, payload))
}
