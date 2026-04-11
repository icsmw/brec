#[macro_use]
mod macros;

use super::{WasmFieldHint, WasmFieldHintId};
use crate::*;
use js_sys::{Array, BigInt, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

const PAYLOAD_FIELD_NAME: &str = "payload";
const BLOCKS_FIELD_NAME: &str = "blocks";

/// Rust <-> JS object conversion contract used by `wasm` helpers.
pub trait WasmObject: Sized {
    /// Converts this value into a JavaScript object representation.
    fn to_wasm_object(&self) -> Result<JsValue, Error>;
    /// Constructs this value from a JavaScript object representation.
    fn from_wasm_object(value: JsValue) -> Result<Self, Error>;
}

/// Schema-driven Rust <-> JS conversion used by payload nested types.
pub trait WasmConvert: Sized {
    fn to_wasm_value(&self) -> Result<JsValue, Error>;
    fn from_wasm_value(value: JsValue) -> Result<Self, Error>;
}

fn get_object(value: JsValue, field: WasmFieldHint) -> Result<js_sys::Object, Error> {
    value
        .dyn_into::<js_sys::Object>()
        .map_err(|_| WasmError::invalid_field(field, "expected object"))
}

fn get_field(obj: &js_sys::Object, name: &str, field: WasmFieldHint) -> Result<JsValue, Error> {
    let key = JsValue::from_str(name);
    let has = Reflect::has(obj, &key)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to access field {name}")))?;
    if !has {
        return Err(Error::Wasm(WasmError::MissingField(name.to_owned())));
    }
    Reflect::get(obj, &key)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to read field {name}")))
}

fn set_field(
    obj: &js_sys::Object,
    name: &str,
    value: &JsValue,
    field: WasmFieldHint,
) -> Result<(), Error> {
    let key = JsValue::from_str(name);
    Reflect::set(obj, &key, value)
        .map_err(|_| WasmError::invalid_field(field, format!("failed to write field {name}")))?;
    Ok(())
}

impl WasmConvert for bool {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        Ok(JsValue::from_bool(*self))
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
        value
            .as_bool()
            .ok_or_else(|| WasmError::invalid_field(WasmFieldHint::Bool, "expected boolean"))
    }
}

impl WasmConvert for String {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        Ok(JsValue::from_str(self))
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
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
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        <u32 as WasmConvert>::to_wasm_value(&self.to_bits())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
        let bits = <u32 as WasmConvert>::from_wasm_value(value)?;
        Ok(f32::from_bits(bits))
    }
}

impl WasmConvert for f64 {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        Ok(BigInt::from(self.to_bits()).into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
        let bits = <u64 as WasmConvert>::from_wasm_value(value).map_err(|_| {
            WasmError::invalid_field(WasmFieldHint::F64, "expected BigInt for f64 bits")
        })?;
        Ok(f64::from_bits(bits))
    }
}

impl<T: WasmConvert> WasmConvert for Vec<T> {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        let arr = Array::new();
        for item in self {
            arr.push(&item.to_wasm_value()?);
        }
        Ok(arr.into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
        if !Array::is_array(&value) {
            return Err(WasmError::invalid_field(
                WasmFieldHint::Vec,
                "expected array",
            ));
        }
        let arr = Array::from(&value);
        let mut out = Vec::with_capacity(arr.length() as usize);
        for idx in 0..arr.length() {
            out.push(T::from_wasm_value(arr.get(idx))?);
        }
        Ok(out)
    }
}

impl<T: WasmConvert> WasmConvert for Option<T> {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        match self {
            Some(v) => v.to_wasm_value(),
            None => Ok(JsValue::NULL),
        }
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
        if value.is_null() || value.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(T::from_wasm_value(value)?))
        }
    }
}

impl<const N: usize> WasmConvert for [u8; N] {
    fn to_wasm_value(&self) -> Result<JsValue, Error> {
        let view = Uint8Array::from(self.as_slice());
        Ok(view.into())
    }

    fn from_wasm_value(value: JsValue) -> Result<Self, Error> {
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
            Error::Wasm(WasmError::InvalidField(
                WasmFieldHintId::Blob.to_string(),
                format!("expected {N} bytes, got {}", bytes.len()),
            ))
        })
    }
}

impl<B: BlockDef + WasmObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + WasmObject>
    PacketDef<B, P, Inner>
{
    /// Converts packet into `{ blocks: Array<{}>, payload: {} | null }`.
    pub fn to_wasm_object(&self) -> Result<JsValue, Error> {
        let obj = js_sys::Object::new();
        let blocks = Array::new();
        for block in self.blocks.iter() {
            blocks.push(&block.to_wasm_object()?);
        }
        set_field(
            &obj,
            BLOCKS_FIELD_NAME,
            &blocks.into(),
            WasmFieldHint::Blocks,
        )?;

        let payload = match self.payload.as_ref() {
            Some(payload) => payload.to_wasm_object()?,
            None => JsValue::NULL,
        };
        set_field(&obj, PAYLOAD_FIELD_NAME, &payload, WasmFieldHint::Payload)?;
        Ok(obj.into())
    }

    /// Parses packet from `{ blocks: Array<{}>, payload: {} | null | undefined }`.
    pub fn from_wasm_object(value: JsValue) -> Result<Self, Error> {
        let obj = get_object(value, WasmFieldHint::Object)
            .map_err(|err| Error::Wasm(WasmError::InvalidObject(err.to_string())))?;

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
            blocks.push(B::from_wasm_object(blocks_arr.get(idx)).map_err(|err| {
                Error::Wasm(WasmError::InvalidField(
                    WasmFieldHintId::Blocks.to_string(),
                    format!("index {idx}: {err}"),
                ))
            })?);
        }

        let payload =
            if Reflect::has(&obj, &JsValue::from_str(PAYLOAD_FIELD_NAME)).map_err(|_| {
                WasmError::invalid_field(WasmFieldHint::Payload, "failed to inspect payload")
            })? {
                let raw =
                    Reflect::get(&obj, &JsValue::from_str(PAYLOAD_FIELD_NAME)).map_err(|_| {
                        WasmError::invalid_field(WasmFieldHint::Payload, "failed to read payload")
                    })?;
                if raw.is_null() || raw.is_undefined() {
                    None
                } else {
                    Some(Inner::from_wasm_object(raw)?)
                }
            } else {
                None
            };

        Ok(Self::new(blocks, payload))
    }

    /// Reads packet bytes and converts to JS object.
    pub fn decode_wasm(
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<JsValue, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        packet.to_wasm_object()
    }

    /// Parses JS object packet and encodes into packet bytes.
    pub fn encode_wasm(
        value: JsValue,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let mut packet = Self::from_wasm_object(value)?;
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
