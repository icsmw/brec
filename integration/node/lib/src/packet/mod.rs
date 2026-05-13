#[macro_use]
mod macros;

use super::{NapiError, NapiFieldHint, NapiFieldHintId, from_unknown as napi_from_unknown};
use brec_consts::*;
use napi::{
    Env, Unknown, ValueType,
    bindgen_prelude::{Array, BigInt, JsObjectValue, JsValue, Object, ToNapiValue},
};

/// Rust <-> JS object conversion contract used by `napi` helpers.
pub trait NapiObject: Sized {
    /// Converts this value into a JavaScript object representation.
    fn to_napi_object<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError>;
    /// Constructs this value from a JavaScript object representation.
    fn from_napi_object(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError>;
}

/// Schema-driven Rust <-> JS conversion used by payload nested types.
pub trait NapiConvert: Sized {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError>;
    fn from_napi_value(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError>;
}

impl_napi_simple!(
    bool => NapiFieldHint::Bool,
    u8 => NapiFieldHint::U8,
    u16 => NapiFieldHint::U16,
    u32 => NapiFieldHint::U32,
    i8 => NapiFieldHint::I8,
    i16 => NapiFieldHint::I16,
    i32 => NapiFieldHint::I32,
    String => NapiFieldHint::String
);

impl_napi_bigint_signed!(
    i64 => (NapiFieldHint::I64, NapiFieldHintId::I64, get_i64),
    i128 => (NapiFieldHint::I128, NapiFieldHintId::I128, get_i128),
);

impl_napi_bigint_unsigned!(
    u64 => (NapiFieldHint::U64, NapiFieldHintId::U64, get_u64),
    u128 => (NapiFieldHint::U128, NapiFieldHintId::U128, get_u128),
);

impl NapiConvert for f32 {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError> {
        NapiConvert::to_napi_value(&self.to_bits(), env)
    }

    fn from_napi_value(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError> {
        let bits = <u32 as NapiConvert>::from_napi_value(env, value)?;
        Ok(f32::from_bits(bits))
    }
}

impl NapiConvert for f64 {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError> {
        let bits = BigInt::from(self.to_bits());
        bits.into_unknown(env)
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::F64, err))
    }

    fn from_napi_value(_env: &Env, value: Unknown<'_>) -> Result<Self, NapiError> {
        let bits = match value
            .get_type()
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::F64, err))?
        {
            ValueType::BigInt => {
                let raw: BigInt = napi_from_unknown(NapiFieldHint::F64, value)?;
                let (sign, bits, lossless) = raw.get_u64();
                if sign || !lossless {
                    return Err(NapiError::InvalidField(
                        NapiFieldHintId::F64.to_string(),
                        "BigInt is out of range for f64".to_owned(),
                    ));
                }
                bits
            }
            other => {
                return Err(NapiError::InvalidField(
                    NapiFieldHintId::F64.to_string(),
                    format!("expected BigInt, got {:?}", other),
                ));
            }
        };
        Ok(f64::from_bits(bits))
    }
}

impl<T: NapiConvert> NapiConvert for Vec<T> {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError> {
        let mut arr = env
            .create_array(self.len() as u32)
            .map_err(|err| NapiError::InvalidObject(err.to_string()))?;
        for (idx, item) in self.iter().enumerate() {
            let value = item.to_napi_value(env)?;
            arr.set(idx as u32, value)
                .map_err(|err| NapiError::invalid_field(NapiFieldHint::Vec, err))?;
        }
        Ok(arr.to_unknown())
    }

    fn from_napi_value(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError> {
        let arr: Array<'_> = napi_from_unknown(NapiFieldHint::Vec, value)
            .map_err(|err| NapiError::InvalidObject(err.to_string()))?;
        let mut out = Vec::with_capacity(arr.len() as usize);
        for idx in 0..arr.len() {
            let raw: Unknown<'_> = arr
                .get(idx)
                .map_err(|err| NapiError::invalid_field(NapiFieldHint::Vec, err))?
                .ok_or_else(|| {
                    NapiError::InvalidField(
                        NapiFieldHintId::Vec.to_string(),
                        "missing element".to_owned(),
                    )
                })?;
            out.push(T::from_napi_value(env, raw)?);
        }
        Ok(out)
    }
}

impl<T: NapiConvert> NapiConvert for Option<T> {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError> {
        match self {
            Some(v) => T::to_napi_value(v, env),
            None => ()
                .into_unknown(env)
                .map_err(|err| NapiError::invalid_field(NapiFieldHint::Option, err)),
        }
    }

    fn from_napi_value(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError> {
        match value
            .get_type()
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Option, err))?
        {
            ValueType::Null | ValueType::Undefined => Ok(None),
            _ => Ok(Some(T::from_napi_value(env, value)?)),
        }
    }
}

impl<const N: usize> NapiConvert for [u8; N] {
    fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, NapiError> {
        let raw: Vec<u8> = self.to_vec();
        <Vec<u8> as NapiConvert>::to_napi_value(&raw, env)
    }

    fn from_napi_value(env: &Env, value: Unknown<'_>) -> Result<Self, NapiError> {
        let raw = <Vec<u8> as NapiConvert>::from_napi_value(env, value)?;
        raw.try_into().map_err(|bytes: Vec<u8>| {
            NapiError::InvalidField(
                NapiFieldHintId::Blob.to_string(),
                format!("expected {} bytes, got {}", N, bytes.len()),
            )
        })
    }
}

/// Converts packet into `{ blocks: Array<{}>, payload?: {} }`.
pub fn to_napi_object<'env, Block: NapiObject, Payload: NapiObject>(
    env: &'env Env,
    blocks: &[Block],
    payload: Option<&Payload>,
) -> Result<Unknown<'env>, NapiError> {
    let mut obj = Object::new(env).map_err(|err| NapiError::InvalidObject(err.to_string()))?;
    let mut js_blocks = env
        .create_array(blocks.len() as u32)
        .map_err(|err| NapiError::invalid_field(NapiFieldHint::Blocks, err))?;
    for (idx, block) in blocks.iter().enumerate() {
        let value = block.to_napi_object(env)?;
        js_blocks
            .set(idx as u32, value)
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Blocks, err))?;
    }
    obj.set_named_property(BLOCKS_FIELD_NAME, js_blocks)
        .map_err(|err| NapiError::invalid_field(NapiFieldHint::Blocks, err))?;
    if let Some(payload) = payload.as_ref() {
        let payload = payload.to_napi_object(env)?;
        obj.set_named_property(PAYLOAD_FIELD_NAME, payload)
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Payload, err))?;
    }
    Ok(obj.to_unknown())
}

/// Parses packet from `{ blocks: Array<{}>, payload?: {} | null | undefined }`.
pub fn from_napi_object<Block: NapiObject, Payload: NapiObject>(
    env: &Env,
    value: Unknown<'_>,
) -> Result<(Vec<Block>, Option<Payload>), NapiError> {
    let obj: Object<'_> = napi_from_unknown(NapiFieldHint::Object, value)
        .map_err(|err| NapiError::InvalidObject(err.to_string()))?;
    let blocks_obj: Array<'_> = obj
        .get_named_property(BLOCKS_FIELD_NAME)
        .map_err(|err| NapiError::invalid_field(NapiFieldHint::Blocks, err))?;
    let blocks_len = blocks_obj.len();
    let mut blocks = Vec::with_capacity(blocks_len as usize);
    for idx in 0..blocks_len {
        let block_val: Unknown<'_> = blocks_obj
            .get(idx)
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Blocks, err))?
            .ok_or_else(|| {
                NapiError::InvalidField(
                    NapiFieldHintId::Blocks.to_string(),
                    format!("missing element at index {}", idx),
                )
            })?;
        blocks.push(Block::from_napi_object(env, block_val)?);
    }

    let payload = if obj
        .has_named_property(PAYLOAD_FIELD_NAME)
        .map_err(|err| NapiError::invalid_field(NapiFieldHint::Payload, err))?
    {
        let raw: Unknown<'_> = obj
            .get_named_property(PAYLOAD_FIELD_NAME)
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Payload, err))?;
        match raw
            .get_type()
            .map_err(|err| NapiError::invalid_field(NapiFieldHint::Payload, err))?
        {
            ValueType::Null | ValueType::Undefined => None,
            _ => Some(Payload::from_napi_object(env, raw)?),
        }
    } else {
        None
    };
    Ok((blocks, payload))
}
