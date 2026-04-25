use super::{
    CSharpError, CSharpFieldHint, CSharpFieldHintId, CSharpObjectMap, CSharpValue, map_get,
    map_has, map_put, new_array, new_object,
};

const PAYLOAD_FIELD_NAME: &str = "payload";
const BLOCKS_FIELD_NAME: &str = "blocks";

/// Rust <-> C# object conversion contract used by `csharp` helpers.
pub trait CSharpObject: Sized {
    /// Converts this value into a C# value representation.
    fn to_csharp_object(&self) -> Result<CSharpValue, CSharpError>;
    /// Constructs this value from a C# value representation.
    fn from_csharp_object(value: CSharpValue) -> Result<Self, CSharpError>;
}

/// Schema-driven Rust <-> C# conversion used by payload nested types.
pub trait CSharpConvert: Sized {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError>;
    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError>;
}

#[inline]
fn value_to_i128(value: &CSharpValue, hint: CSharpFieldHint) -> Result<i128, CSharpError> {
    match value {
        CSharpValue::I8(v) => Ok(*v as i128),
        CSharpValue::I16(v) => Ok(*v as i128),
        CSharpValue::I32(v) => Ok(*v as i128),
        CSharpValue::I64(v) => Ok(*v as i128),
        CSharpValue::I128(v) => Ok(*v),
        CSharpValue::U8(v) => Ok(*v as i128),
        CSharpValue::U16(v) => Ok(*v as i128),
        CSharpValue::U32(v) => Ok(*v as i128),
        CSharpValue::U64(v) => Ok(*v as i128),
        CSharpValue::U128(v) => i128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "unsigned value is out of range")),
        other => Err(CSharpError::invalid_field(
            hint,
            format!("expected integer, got {other:?}"),
        )),
    }
}

#[inline]
fn value_to_u128(value: &CSharpValue, hint: CSharpFieldHint) -> Result<u128, CSharpError> {
    match value {
        CSharpValue::U8(v) => Ok(*v as u128),
        CSharpValue::U16(v) => Ok(*v as u128),
        CSharpValue::U32(v) => Ok(*v as u128),
        CSharpValue::U64(v) => Ok(*v as u128),
        CSharpValue::U128(v) => Ok(*v),
        CSharpValue::I8(v) => u128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "negative value is not allowed")),
        CSharpValue::I16(v) => u128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "negative value is not allowed")),
        CSharpValue::I32(v) => u128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "negative value is not allowed")),
        CSharpValue::I64(v) => u128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "negative value is not allowed")),
        CSharpValue::I128(v) => u128::try_from(*v)
            .map_err(|_| CSharpError::invalid_field(hint, "negative value is not allowed")),
        other => Err(CSharpError::invalid_field(
            hint,
            format!("expected integer, got {other:?}"),
        )),
    }
}

impl CSharpConvert for bool {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        Ok(CSharpValue::Bool(*self))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Bool(v) => Ok(v),
            CSharpValue::Null => Err(CSharpError::invalid_field(
                CSharpFieldHint::Bool,
                "null is not allowed",
            )),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::Bool,
                format!("expected bool, got {other:?}"),
            )),
        }
    }
}

impl CSharpConvert for String {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        Ok(CSharpValue::String(self.clone()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::String(v) => Ok(v),
            CSharpValue::Null => Err(CSharpError::invalid_field(
                CSharpFieldHint::String,
                "null is not allowed",
            )),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::String,
                format!("expected string, got {other:?}"),
            )),
        }
    }
}

macro_rules! impl_csharp_int_signed {
    ($($ty:ty => $hint:expr),* $(,)?) => {
        $(
            impl CSharpConvert for $ty {
                fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
                    Ok(match $hint {
                        CSharpFieldHint::I8 => CSharpValue::I8(*self as i8),
                        CSharpFieldHint::I16 => CSharpValue::I16(*self as i16),
                        CSharpFieldHint::I32 => CSharpValue::I32(*self as i32),
                        CSharpFieldHint::I64 => CSharpValue::I64(*self as i64),
                        CSharpFieldHint::I128 => CSharpValue::I128(*self as i128),
                        _ => unreachable!(),
                    })
                }

                fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
                    let raw = value_to_i128(&value, $hint)?;
                    <$ty>::try_from(raw)
                        .map_err(|_| CSharpError::invalid_field($hint, "value is out of range"))
                }
            }
        )*
    };
}

macro_rules! impl_csharp_int_unsigned {
    ($($ty:ty => $hint:expr),* $(,)?) => {
        $(
            impl CSharpConvert for $ty {
                fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
                    Ok(match $hint {
                        CSharpFieldHint::U8 => CSharpValue::U8(*self as u8),
                        CSharpFieldHint::U16 => CSharpValue::U16(*self as u16),
                        CSharpFieldHint::U32 => CSharpValue::U32(*self as u32),
                        CSharpFieldHint::U64 => CSharpValue::U64(*self as u64),
                        CSharpFieldHint::U128 => CSharpValue::U128(*self as u128),
                        _ => unreachable!(),
                    })
                }

                fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
                    let raw = value_to_u128(&value, $hint)?;
                    <$ty>::try_from(raw)
                        .map_err(|_| CSharpError::invalid_field($hint, "value is out of range"))
                }
            }
        )*
    };
}

impl_csharp_int_signed!(
    i8 => CSharpFieldHint::I8,
    i16 => CSharpFieldHint::I16,
    i32 => CSharpFieldHint::I32,
    i64 => CSharpFieldHint::I64,
    i128 => CSharpFieldHint::I128,
);

impl_csharp_int_unsigned!(
    u8 => CSharpFieldHint::U8,
    u16 => CSharpFieldHint::U16,
    u32 => CSharpFieldHint::U32,
    u64 => CSharpFieldHint::U64,
    u128 => CSharpFieldHint::U128,
);

impl CSharpConvert for f32 {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        Ok(CSharpValue::F32Bits(self.to_bits()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        let bits = match value {
            CSharpValue::F32Bits(bits) => bits,
            CSharpValue::U32(bits) => bits,
            CSharpValue::Null => {
                return Err(CSharpError::invalid_field(
                    CSharpFieldHint::U32,
                    "null is not allowed",
                ));
            }
            other => {
                return Err(CSharpError::invalid_field(
                    CSharpFieldHint::U32,
                    format!("expected f32 bits, got {other:?}"),
                ));
            }
        };
        Ok(f32::from_bits(bits))
    }
}

impl CSharpConvert for f64 {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        Ok(CSharpValue::F64Bits(self.to_bits()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        let bits = match value {
            CSharpValue::F64Bits(bits) => bits,
            CSharpValue::U64(bits) => bits,
            CSharpValue::Null => {
                return Err(CSharpError::invalid_field(
                    CSharpFieldHint::F64,
                    "null is not allowed",
                ));
            }
            other => {
                return Err(CSharpError::invalid_field(
                    CSharpFieldHint::F64,
                    format!("expected f64 bits, got {other:?}"),
                ));
            }
        };
        Ok(f64::from_bits(bits))
    }
}

impl<T: CSharpConvert> CSharpConvert for Vec<T> {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        let mut out = new_array(self.len());
        for item in self {
            super::list_add(&mut out, item.to_csharp_value()?)?;
        }
        Ok(CSharpValue::Array(out))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for elem in arr {
                    out.push(T::from_csharp_value(elem)?);
                }
                Ok(out)
            }
            CSharpValue::Null => Err(CSharpError::invalid_field(
                CSharpFieldHint::Vec,
                "null is not allowed",
            )),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::Vec,
                format!("expected array, got {other:?}"),
            )),
        }
    }
}

impl<T: CSharpConvert> CSharpConvert for Option<T> {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        match self {
            Some(v) => v.to_csharp_value(),
            None => Ok(CSharpValue::Null),
        }
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Null => Ok(None),
            other => Ok(Some(T::from_csharp_value(other)?)),
        }
    }
}

impl<const N: usize> CSharpConvert for [u8; N] {
    fn to_csharp_value(&self) -> Result<CSharpValue, CSharpError> {
        Ok(CSharpValue::Bytes(self.to_vec()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, CSharpError> {
        match value {
            CSharpValue::Bytes(bytes) => bytes.try_into().map_err(|bytes: Vec<u8>| {
                CSharpError::InvalidField(
                    CSharpFieldHintId::Blob.to_string(),
                    format!("expected {N} bytes, got {}", bytes.len()),
                )
            }),
            CSharpValue::Array(arr) => {
                if arr.len() != N {
                    return Err(CSharpError::InvalidField(
                        CSharpFieldHintId::Blob.to_string(),
                        format!("expected {N} bytes, got {}", arr.len()),
                    ));
                }
                let mut out = [0_u8; N];
                for (idx, item) in arr.into_iter().enumerate() {
                    out[idx] = <u8 as CSharpConvert>::from_csharp_value(item)
                        .map_err(|err| CSharpError::invalid_field_name(idx.to_string(), err))?;
                }
                Ok(out)
            }
            CSharpValue::Null => Err(CSharpError::invalid_field(
                CSharpFieldHint::Blob,
                "null is not allowed",
            )),
            other => Err(CSharpError::invalid_field(
                CSharpFieldHint::Blob,
                format!("expected bytes, got {other:?}"),
            )),
        }
    }
}

/// Converts packet into `{ blocks: Array<{}>, payload: {} | null }` using `CSharpValue` ABI.
pub fn to_csharp_object<Block: CSharpObject, Payload: CSharpObject>(
    blocks: &[Block],
    payload: Option<&Payload>,
) -> Result<CSharpValue, CSharpError> {
    let mut obj = new_object();
    let mut blocks_arr = new_array(blocks.len());
    for block in blocks {
        super::list_add(&mut blocks_arr, block.to_csharp_object()?)?;
    }
    map_put(&mut obj, BLOCKS_FIELD_NAME, CSharpValue::Array(blocks_arr))?;

    let payload_value = match payload {
        Some(payload) => payload.to_csharp_object()?,
        None => CSharpValue::Null,
    };
    map_put(&mut obj, PAYLOAD_FIELD_NAME, payload_value)?;
    Ok(CSharpValue::Object(obj))
}

/// Parses packet from `{ blocks: Array<{}>, payload: {} | null }` using `CSharpValue` ABI.
pub fn from_csharp_object<Block: CSharpObject, Payload: CSharpObject>(
    value: CSharpValue,
) -> Result<(Vec<Block>, Option<Payload>), CSharpError> {
    let mut obj: CSharpObjectMap = super::from_value_name("object", value)
        .map_err(|err| CSharpError::InvalidObject(err.to_string()))?;

    let blocks_raw = super::map_take(&mut obj, BLOCKS_FIELD_NAME)?;
    let blocks_arr: Vec<CSharpValue> = super::from_value_name(BLOCKS_FIELD_NAME, blocks_raw)
        .map_err(|err| CSharpError::invalid_field(CSharpFieldHint::Blocks, err))?;
    let mut blocks = Vec::with_capacity(blocks_arr.len());
    for (idx, block_val) in blocks_arr.into_iter().enumerate() {
        blocks.push(Block::from_csharp_object(block_val).map_err(|err| {
            CSharpError::InvalidField(
                CSharpFieldHintId::Blocks.to_string(),
                format!("index {idx}: {err}"),
            )
        })?);
    }

    let payload = match map_has(&obj, PAYLOAD_FIELD_NAME)? {
        false => None,
        true => {
            let raw = map_get(&obj, PAYLOAD_FIELD_NAME)?;
            match raw {
                CSharpValue::Null => None,
                other => Some(Payload::from_csharp_object(other)?),
            }
        }
    };

    Ok((blocks, payload))
}
