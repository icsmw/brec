use super::{CSharpError, CSharpFieldHint, CSharpFieldHintId, CSharpValue};
use crate::*;

const PAYLOAD_FIELD_NAME: &str = "payload";
const BLOCKS_FIELD_NAME: &str = "blocks";

/// Rust <-> C# object conversion contract used by `csharp` helpers.
pub trait CSharpObject: Sized {
    /// Converts this value into a C# value representation.
    fn to_csharp_object(&self) -> Result<CSharpValue, Error>;
    /// Constructs this value from a C# value representation.
    fn from_csharp_object(value: CSharpValue) -> Result<Self, Error>;
}

/// Schema-driven Rust <-> C# conversion used by payload nested types.
pub trait CSharpConvert: Sized {
    fn to_csharp_value(&self) -> Result<CSharpValue, Error>;
    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error>;
}

#[inline]
fn value_to_i128(value: &CSharpValue, hint: CSharpFieldHint) -> Result<i128, Error> {
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
fn value_to_u128(value: &CSharpValue, hint: CSharpFieldHint) -> Result<u128, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        Ok(CSharpValue::Bool(*self))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        Ok(CSharpValue::String(self.clone()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
                fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
                    Ok(match $hint {
                        CSharpFieldHint::I8 => CSharpValue::I8(*self as i8),
                        CSharpFieldHint::I16 => CSharpValue::I16(*self as i16),
                        CSharpFieldHint::I32 => CSharpValue::I32(*self as i32),
                        CSharpFieldHint::I64 => CSharpValue::I64(*self as i64),
                        CSharpFieldHint::I128 => CSharpValue::I128(*self as i128),
                        _ => unreachable!(),
                    })
                }

                fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
                fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
                    Ok(match $hint {
                        CSharpFieldHint::U8 => CSharpValue::U8(*self as u8),
                        CSharpFieldHint::U16 => CSharpValue::U16(*self as u16),
                        CSharpFieldHint::U32 => CSharpValue::U32(*self as u32),
                        CSharpFieldHint::U64 => CSharpValue::U64(*self as u64),
                        CSharpFieldHint::U128 => CSharpValue::U128(*self as u128),
                        _ => unreachable!(),
                    })
                }

                fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        Ok(CSharpValue::F32Bits(self.to_bits()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        Ok(CSharpValue::F64Bits(self.to_bits()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        let mut out = super::new_array(self.len());
        for item in self {
            super::list_add(&mut out, item.to_csharp_value()?)?;
        }
        Ok(CSharpValue::Array(out))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
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
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        match self {
            Some(v) => v.to_csharp_value(),
            None => Ok(CSharpValue::Null),
        }
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
        match value {
            CSharpValue::Null => Ok(None),
            other => Ok(Some(T::from_csharp_value(other)?)),
        }
    }
}

impl<const N: usize> CSharpConvert for [u8; N] {
    fn to_csharp_value(&self) -> Result<CSharpValue, Error> {
        Ok(CSharpValue::Bytes(self.to_vec()))
    }

    fn from_csharp_value(value: CSharpValue) -> Result<Self, Error> {
        match value {
            CSharpValue::Bytes(bytes) => bytes.try_into().map_err(|bytes: Vec<u8>| {
                Error::CSharp(CSharpError::InvalidField(
                    CSharpFieldHintId::Blob.to_string(),
                    format!("expected {N} bytes, got {}", bytes.len()),
                ))
            }),
            CSharpValue::Array(arr) => {
                if arr.len() != N {
                    return Err(Error::CSharp(CSharpError::InvalidField(
                        CSharpFieldHintId::Blob.to_string(),
                        format!("expected {N} bytes, got {}", arr.len()),
                    )));
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

impl<B: BlockDef + CSharpObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + CSharpObject>
    PacketDef<B, P, Inner>
{
    /// Converts packet into `{ blocks: Array<{}>, payload: {} | null }` using `CSharpValue` ABI.
    pub fn to_csharp_object(&self) -> Result<CSharpValue, Error> {
        let mut obj = super::new_object();
        let mut blocks = super::new_array(self.blocks.len());
        for block in &self.blocks {
            super::list_add(&mut blocks, block.to_csharp_object()?)?;
        }
        super::map_put(&mut obj, BLOCKS_FIELD_NAME, CSharpValue::Array(blocks))?;

        let payload = match self.payload.as_ref() {
            Some(payload) => payload.to_csharp_object()?,
            None => CSharpValue::Null,
        };
        super::map_put(&mut obj, PAYLOAD_FIELD_NAME, payload)?;
        Ok(CSharpValue::Object(obj))
    }

    /// Parses packet from `{ blocks: Array<{}>, payload: {} | null }` using `CSharpValue` ABI.
    pub fn from_csharp_object(value: CSharpValue) -> Result<Self, Error> {
        let mut obj: super::CSharpObjectMap = super::from_value_name("object", value)
            .map_err(|err| Error::CSharp(CSharpError::InvalidObject(err.to_string())))?;

        let blocks_raw = super::map_take(&mut obj, BLOCKS_FIELD_NAME)?;
        let blocks_arr: Vec<CSharpValue> = super::from_value_name(BLOCKS_FIELD_NAME, blocks_raw)
            .map_err(|err| CSharpError::invalid_field(CSharpFieldHint::Blocks, err))?;
        let mut blocks = Vec::with_capacity(blocks_arr.len());
        for (idx, block_val) in blocks_arr.into_iter().enumerate() {
            blocks.push(B::from_csharp_object(block_val).map_err(|err| {
                Error::CSharp(CSharpError::InvalidField(
                    CSharpFieldHintId::Blocks.to_string(),
                    format!("index {idx}: {err}"),
                ))
            })?);
        }

        let payload = match obj.remove(PAYLOAD_FIELD_NAME) {
            None | Some(CSharpValue::Null) => None,
            Some(other) => Some(Inner::from_csharp_object(other)?),
        };

        Ok(Self::new(blocks, payload))
    }

    /// Reads packet bytes and converts to `CSharpValue` object.
    pub fn decode_csharp(
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<CSharpValue, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        packet.to_csharp_object()
    }

    /// Parses `CSharpValue` packet object and encodes into packet bytes.
    pub fn encode_csharp(
        value: CSharpValue,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let mut packet = Self::from_csharp_object(value)?;
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
