#[macro_use]
mod macros;

use super::{WasmError, WasmFieldHint, WasmFieldHintId};
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
        let arr = Array::new_with_length(self.len() as u32);
        for (idx, item) in self.iter().enumerate() {
            arr.set(idx as u32, item.to_wasm_value()?);
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
        let blocks = Array::new_with_length(self.blocks.len() as u32);
        for (idx, block) in self.blocks.iter().enumerate() {
            blocks.set(idx as u32, block.to_wasm_object()?);
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

#[cfg(all(test, target_arch = "wasm32", feature = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn bool_and_string_roundtrip() {
        let b = <bool as WasmConvert>::from_wasm_value(
            <bool as WasmConvert>::to_wasm_value(&true).expect("bool to js"),
        )
        .expect("bool from js");
        assert!(b);

        let s = "hello wasm".to_owned();
        let back = <String as WasmConvert>::from_wasm_value(
            <String as WasmConvert>::to_wasm_value(&s).expect("string to js"),
        )
        .expect("string from js");
        assert_eq!(back, s);
    }

    #[wasm_bindgen_test]
    fn integer_number_roundtrip_and_validation() {
        let v_u16: u16 = 6553;
        let v_i32: i32 = -123456;

        let u16_back = <u16 as WasmConvert>::from_wasm_value(
            <u16 as WasmConvert>::to_wasm_value(&v_u16).expect("u16 to js"),
        )
        .expect("u16 from js");
        assert_eq!(u16_back, v_u16);

        let i32_back = <i32 as WasmConvert>::from_wasm_value(
            <i32 as WasmConvert>::to_wasm_value(&v_i32).expect("i32 to js"),
        )
        .expect("i32 from js");
        assert_eq!(i32_back, v_i32);

        let frac_err =
            <u8 as WasmConvert>::from_wasm_value(JsValue::from_f64(10.5)).expect_err("fraction");
        assert!(frac_err.to_string().contains("finite integer"));

        let range_err =
            <u8 as WasmConvert>::from_wasm_value(JsValue::from_f64(300.0)).expect_err("range");
        assert!(range_err.to_string().contains("out of range"));
    }

    #[wasm_bindgen_test]
    fn bigint_roundtrip_and_rejection_for_wide_ints() {
        let values = [
            i64::MIN as i128,
            i64::MAX as i128,
            i128::MIN + 12345,
            i128::MAX - 12345,
        ];
        for v in values {
            let js = <i128 as WasmConvert>::to_wasm_value(&v).expect("i128 to js");
            assert!(js.is_bigint());
            let back = <i128 as WasmConvert>::from_wasm_value(js).expect("i128 from js");
            assert_eq!(back, v);
        }

        let max_u = u128::MAX;
        let js_u = <u128 as WasmConvert>::to_wasm_value(&max_u).expect("u128 to js");
        assert!(js_u.is_bigint());
        let back_u = <u128 as WasmConvert>::from_wasm_value(js_u).expect("u128 from js");
        assert_eq!(back_u, max_u);

        let not_bigint = <u64 as WasmConvert>::from_wasm_value(JsValue::from_f64(42.0))
            .expect_err("u64 must reject Number");
        assert!(not_bigint.to_string().contains("expected BigInt"));
    }

    #[wasm_bindgen_test]
    fn float_roundtrip_preserves_bits() {
        let f32_values = [0.0f32, -0.0f32, 1.5f32, -12.25f32, f32::MIN_POSITIVE];
        for v in f32_values {
            let back = <f32 as WasmConvert>::from_wasm_value(
                <f32 as WasmConvert>::to_wasm_value(&v).expect("f32 to js"),
            )
            .expect("f32 from js");
            assert_eq!(back.to_bits(), v.to_bits());
        }

        let f64_values = [0.0f64, -0.0f64, 1.5f64, -12.25f64, f64::MIN_POSITIVE];
        for v in f64_values {
            let js = <f64 as WasmConvert>::to_wasm_value(&v).expect("f64 to js");
            assert!(js.is_bigint());
            let back = <f64 as WasmConvert>::from_wasm_value(js).expect("f64 from js");
            assert_eq!(back.to_bits(), v.to_bits());
        }

        let wrong_f64 =
            <f64 as WasmConvert>::from_wasm_value(JsValue::from_f64(1.25)).expect_err("f64 bits");
        assert!(
            wrong_f64
                .to_string()
                .contains("expected BigInt for f64 bits")
        );
    }

    #[wasm_bindgen_test]
    fn vec_option_and_blob_roundtrip_and_errors() {
        let vec_value = vec![1u16, 2, 3, 65535];
        let vec_back = <Vec<u16> as WasmConvert>::from_wasm_value(
            <Vec<u16> as WasmConvert>::to_wasm_value(&vec_value).expect("vec to js"),
        )
        .expect("vec from js");
        assert_eq!(vec_back, vec_value);

        let vec_err = <Vec<u8> as WasmConvert>::from_wasm_value(JsValue::from_str("not array"))
            .expect_err("vec reject non-array");
        assert!(vec_err.to_string().contains("expected array"));

        let some = Some(9u8);
        let some_back = <Option<u8> as WasmConvert>::from_wasm_value(
            <Option<u8> as WasmConvert>::to_wasm_value(&some).expect("opt to js"),
        )
        .expect("opt from js");
        assert_eq!(some_back, some);

        let none_back = <Option<u8> as WasmConvert>::from_wasm_value(JsValue::UNDEFINED)
            .expect("undefined -> none");
        assert_eq!(none_back, None);
        let none_js = <Option<u8> as WasmConvert>::to_wasm_value(&None).expect("none to js");
        assert!(none_js.is_null());

        let blob = [1u8, 2, 3, 4];
        let blob_back = <[u8; 4] as WasmConvert>::from_wasm_value(
            <[u8; 4] as WasmConvert>::to_wasm_value(&blob).expect("blob to js"),
        )
        .expect("blob from js");
        assert_eq!(blob_back, blob);

        let array = js_sys::Array::new();
        array.push(&JsValue::from_f64(7.0));
        array.push(&JsValue::from_f64(8.0));
        array.push(&JsValue::from_f64(9.0));
        array.push(&JsValue::from_f64(10.0));
        let from_array =
            <[u8; 4] as WasmConvert>::from_wasm_value(array.into()).expect("array -> [u8;4]");
        assert_eq!(from_array, [7, 8, 9, 10]);

        let short = js_sys::Array::new();
        short.push(&JsValue::from_f64(1.0));
        let blob_err =
            <[u8; 4] as WasmConvert>::from_wasm_value(short.into()).expect_err("len mismatch");
        assert!(blob_err.to_string().contains("expected 4 bytes"));
    }

    mod packet_object_flow {
        use super::*;

        const ID_FIELD_NAME: &str = "id";

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct WBlock {
            id: u8,
        }

        impl WBlock {
            fn new(id: u8) -> Self {
                Self { id }
            }
        }

        impl WasmObject for WBlock {
            fn to_wasm_object(&self) -> Result<JsValue, Error> {
                let obj = js_sys::Object::new();
                let id = <u8 as WasmConvert>::to_wasm_value(&self.id)?;
                set_field(&obj, ID_FIELD_NAME, &id, WasmFieldHint::U8)?;
                Ok(obj.into())
            }

            fn from_wasm_object(value: JsValue) -> Result<Self, Error> {
                let obj = get_object(value, WasmFieldHint::Object)
                    .map_err(|err| Error::Wasm(WasmError::InvalidObject(err.to_string())))?;
                let id_raw = get_field(&obj, ID_FIELD_NAME, WasmFieldHint::U8)?;
                let id = <u8 as WasmConvert>::from_wasm_value(id_raw)?;
                Ok(Self { id })
            }
        }

        impl Size for WBlock {
            fn size(&self) -> u64 {
                1
            }
        }

        impl WriteVectoredTo for WBlock {
            fn slices(&self) -> std::io::Result<IoSlices<'_>> {
                Err(std::io::Error::other("wasm packet test block"))
            }
        }

        impl WriteTo for WBlock {
            fn write<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<usize> {
                Err(std::io::Error::other("wasm packet test block"))
            }

            fn write_all<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<()> {
                Err(std::io::Error::other("wasm packet test block"))
            }
        }

        impl TryReadFromBuffered for WBlock {
            fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
                Err(Error::Test)
            }
        }

        impl TryReadFrom for WBlock {
            fn try_read<T: std::io::Read + std::io::Seek>(
                _: &mut T,
            ) -> Result<ReadStatus<Self>, Error> {
                Err(Error::Test)
            }
        }

        impl ReadFrom for WBlock {
            fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error> {
                Err(Error::Test)
            }
        }

        impl ReadBlockFrom for WBlock {
            fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error> {
                Err(Error::Test)
            }
        }

        impl ReadBlockFromSlice for WBlock {
            fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
            where
                Self: 'a + Sized,
            {
                Err(Error::Test)
            }
        }

        impl BlockDef for WBlock {}

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct WPayload {
            id: u8,
        }

        impl WPayload {
            fn new(id: u8) -> Self {
                Self { id }
            }
        }

        impl WasmObject for WPayload {
            fn to_wasm_object(&self) -> Result<JsValue, Error> {
                let obj = js_sys::Object::new();
                let id = <u8 as WasmConvert>::to_wasm_value(&self.id)?;
                set_field(&obj, ID_FIELD_NAME, &id, WasmFieldHint::U8)?;
                Ok(obj.into())
            }

            fn from_wasm_object(value: JsValue) -> Result<Self, Error> {
                let obj = get_object(value, WasmFieldHint::Object)
                    .map_err(|err| Error::Wasm(WasmError::InvalidObject(err.to_string())))?;
                let id_raw = get_field(&obj, ID_FIELD_NAME, WasmFieldHint::U8)?;
                let id = <u8 as WasmConvert>::from_wasm_value(id_raw)?;
                Ok(Self { id })
            }
        }

        impl PayloadSchema for WPayload {
            type Context<'a> = DefaultPayloadContext;
        }

        impl WriteVectoredMutTo for WPayload {
            fn slices(&mut self, _: &mut Self::Context<'_>) -> std::io::Result<IoSlices<'_>> {
                Err(std::io::Error::other("wasm packet test payload"))
            }
        }

        impl WriteMutTo for WPayload {
            fn write<T: std::io::Write>(
                &mut self,
                _: &mut T,
                _: &mut Self::Context<'_>,
            ) -> std::io::Result<usize> {
                Err(std::io::Error::other("wasm packet test payload"))
            }

            fn write_all<T: std::io::Write>(
                &mut self,
                _: &mut T,
                _: &mut Self::Context<'_>,
            ) -> std::io::Result<()> {
                Err(std::io::Error::other("wasm packet test payload"))
            }
        }

        impl PayloadSignature for WPayload {
            fn sig(&self) -> ByteBlock {
                ByteBlock::Len4([0, 0, 0, self.id])
            }
        }

        impl PayloadEncodeReferred for WPayload {
            fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
                Ok(None)
            }
        }

        impl PayloadHooks for WPayload {}

        impl PayloadEncode for WPayload {
            fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
                Ok(vec![self.id])
            }
        }

        impl PayloadCrc for WPayload {
            fn crc(&self, _: &mut Self::Context<'_>) -> std::io::Result<ByteBlock> {
                Ok(ByteBlock::Len4([0, 0, 0, self.id]))
            }

            fn crc_size() -> usize {
                4
            }
        }

        impl PayloadSize for WPayload {
            fn size(&self, _: &mut Self::Context<'_>) -> std::io::Result<u64> {
                Ok(1)
            }
        }

        impl PayloadInnerDef for WPayload {}

        impl TryExtractPayloadFromBuffered<WPayload> for WPayload {
            fn try_read<B: std::io::BufRead>(
                _: &mut B,
                _: &PayloadHeader,
                _: &mut <WPayload as PayloadSchema>::Context<'_>,
            ) -> Result<ReadStatus<WPayload>, Error> {
                Err(Error::Test)
            }
        }

        impl TryExtractPayloadFrom<WPayload> for WPayload {
            fn try_read<B: std::io::Read + std::io::Seek>(
                _: &mut B,
                _: &PayloadHeader,
                _: &mut <WPayload as PayloadSchema>::Context<'_>,
            ) -> Result<ReadStatus<WPayload>, Error> {
                Err(Error::Test)
            }
        }

        impl ExtractPayloadFrom<WPayload> for WPayload {
            fn read<B: std::io::Read>(
                _: &mut B,
                _: &PayloadHeader,
                _: &mut <WPayload as PayloadSchema>::Context<'_>,
            ) -> Result<WPayload, Error> {
                Err(Error::Test)
            }
        }

        impl PayloadDef<WPayload> for WPayload {}

        type WPacket = PacketDef<WBlock, WPayload, WPayload>;

        fn read_id(obj: &js_sys::Object, field: &str) -> Option<u8> {
            js_sys::Reflect::get(obj, &JsValue::from_str(field))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|v| v as u8)
        }

        #[wasm_bindgen_test]
        fn packet_object_roundtrip_via_js_shape() {
            let packet = WPacket::new(vec![WBlock::new(7)], Some(WPayload::new(11)));

            let js = packet.to_wasm_object().expect("to_wasm_object");
            let root = from_value::<js_sys::Object>(WasmFieldHint::Object, js.clone())
                .expect("root object");
            let blocks = js_sys::Reflect::get(&root, &JsValue::from_str("blocks")).expect("blocks");
            let blocks =
                from_value::<js_sys::Array>(WasmFieldHint::Blocks, blocks).expect("blocks array");
            assert_eq!(blocks.length(), 1);
            let first_block = from_value::<js_sys::Object>(WasmFieldHint::Object, blocks.get(0))
                .expect("first block");
            assert_eq!(read_id(&first_block, ID_FIELD_NAME), Some(7));
            let payload =
                js_sys::Reflect::get(&root, &JsValue::from_str("payload")).expect("payload");
            let payload = from_value::<js_sys::Object>(WasmFieldHint::Payload, payload)
                .expect("payload object");
            assert_eq!(read_id(&payload, ID_FIELD_NAME), Some(11));

            let back = WPacket::from_wasm_object(js).expect("from_wasm_object");
            assert_eq!(back.blocks.len(), 1);
            assert_eq!(back.payload.as_ref().map(|p| p.id), Some(11));
            assert_eq!(back.blocks.first().map(|b| b.id), Some(7));
        }

        #[wasm_bindgen_test]
        fn packet_object_rejects_invalid_shapes() {
            let empty = js_sys::Object::new();
            let err = WPacket::from_wasm_object(empty.into())
                .err()
                .expect("missing blocks must fail");
            assert!(err.to_string().contains("Missing field: blocks"));

            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("blocks"),
                &JsValue::from_str("wrong"),
            )
            .expect("set blocks");
            let err = WPacket::from_wasm_object(obj.into())
                .err()
                .expect("wrong blocks must fail");
            assert!(err.to_string().contains("expected array for blocks"));
        }

        #[wasm_bindgen_test]
        fn packet_object_optional_payload_and_decode_encode_paths() {
            let mut packet = WPacket::new(vec![], None);
            let js = packet.to_wasm_object().expect("to_wasm_object");
            let root =
                from_value::<js_sys::Object>(WasmFieldHint::Object, js.clone()).expect("root");
            let payload =
                js_sys::Reflect::get(&root, &JsValue::from_str("payload")).expect("payload");
            assert!(payload.is_null());

            let mut bytes = Vec::new();
            let mut ctx = default_payload_context();
            packet.write_all(&mut bytes, &mut ctx).expect("write_all");

            let decoded = WPacket::decode_wasm(&bytes, &mut ctx).expect("decode_wasm");
            let decoded_root =
                from_value::<js_sys::Object>(WasmFieldHint::Object, decoded.clone()).expect("obj");
            let decoded_blocks =
                js_sys::Reflect::get(&decoded_root, &JsValue::from_str("blocks")).expect("blocks");
            let decoded_blocks =
                from_value::<js_sys::Array>(WasmFieldHint::Blocks, decoded_blocks).expect("array");
            assert_eq!(decoded_blocks.length(), 0);
            let decoded_payload =
                js_sys::Reflect::get(&decoded_root, &JsValue::from_str("payload"))
                    .expect("payload");
            assert!(decoded_payload.is_null());

            let mut reencoded = Vec::new();
            WPacket::encode_wasm(decoded, &mut reencoded, &mut ctx).expect("encode_wasm");
            assert_eq!(reencoded, bytes);
        }

        #[wasm_bindgen_test]
        fn packet_object_reports_block_index_and_payload_reflect_errors() {
            let obj = js_sys::Object::new();
            let blocks = js_sys::Array::new();
            blocks.push(&js_sys::Object::new().into());
            js_sys::Reflect::set(&obj, &JsValue::from_str("blocks"), &blocks.into())
                .expect("set blocks");
            let err = WPacket::from_wasm_object(obj.into())
                .err()
                .expect("invalid block must fail");
            assert!(err.to_string().contains("index 0"));
            assert!(err.to_string().contains("Missing field: id"));

            let proxy_has = js_sys::eval(
                r#"(() => {
                    const target = { blocks: [] };
                    return new Proxy(target, {
                        has(t, p) {
                            if (p === "payload") { throw new Error("has_fail"); }
                            return Reflect.has(t, p);
                        },
                    });
                })()"#,
            )
            .expect("proxy has");
            let err = WPacket::from_wasm_object(proxy_has)
                .err()
                .expect("payload has failure");
            assert!(err.to_string().contains("failed to inspect payload"));

            let proxy_get = js_sys::eval(
                r#"(() => {
                    const target = { blocks: [], payload: {} };
                    return new Proxy(target, {
                        has(t, p) { return Reflect.has(t, p); },
                        get(t, p) {
                            if (p === "payload") { throw new Error("get_fail"); }
                            return Reflect.get(t, p);
                        },
                    });
                })()"#,
            )
            .expect("proxy get");
            let err = WPacket::from_wasm_object(proxy_get)
                .err()
                .expect("payload get failure");
            assert!(err.to_string().contains("failed to read payload"));
        }

        #[wasm_bindgen_test]
        fn packet_object_support_types_trait_methods_are_exercised() {
            let block = WBlock::new(5);
            assert_eq!(block.size(), 1);
            assert!(block.slices().is_err());
            assert!(
                block
                    .write(&mut std::io::Cursor::new(Vec::<u8>::new()))
                    .is_err()
            );
            assert!(
                block
                    .write_all(&mut std::io::Cursor::new(Vec::<u8>::new()))
                    .is_err()
            );
            assert!(
                <WBlock as TryReadFromBuffered>::try_read(&mut std::io::Cursor::new(
                    Vec::<u8>::new()
                ))
                .is_err()
            );
            assert!(
                <WBlock as TryReadFrom>::try_read(&mut std::io::Cursor::new(Vec::<u8>::new()))
                    .is_err()
            );
            assert!(
                <WBlock as ReadFrom>::read(&mut std::io::Cursor::new(Vec::<u8>::new())).is_err()
            );
            assert!(
                <WBlock as ReadBlockFrom>::read(&mut std::io::Cursor::new(Vec::<u8>::new()), false)
                    .is_err()
            );
            assert!(<WBlock as ReadBlockFromSlice>::read_from_slice(&[], false).is_err());

            let mut payload = WPayload::new(9);
            let mut ctx = default_payload_context();
            assert!(payload.slices(&mut ctx).is_err());
            assert!(
                payload
                    .write(&mut std::io::Cursor::new(Vec::<u8>::new()), &mut ctx)
                    .is_err()
            );
            assert!(
                payload
                    .write_all(&mut std::io::Cursor::new(Vec::<u8>::new()), &mut ctx)
                    .is_err()
            );
            assert_eq!(payload.sig(), ByteBlock::Len4([0, 0, 0, 9]));
            assert_eq!(
                <WPayload as PayloadEncode>::encode(&payload, &mut ctx).expect("encode"),
                vec![9]
            );
            assert_eq!(
                <WPayload as PayloadEncodeReferred>::encode(&payload, &mut ctx).expect("referred"),
                None
            );
            assert_eq!(
                payload.crc(&mut ctx).expect("crc"),
                ByteBlock::Len4([0, 0, 0, 9])
            );
            assert_eq!(WPayload::crc_size(), 4);
            assert_eq!(payload.size(&mut ctx).expect("size"), 1);

            let header = PayloadHeader {
                sig: ByteBlock::Len4([0, 0, 0, 1]),
                crc: ByteBlock::Len4([0, 0, 0, 2]),
                len: 0,
            };
            assert!(
                <WPayload as TryExtractPayloadFromBuffered<WPayload>>::try_read(
                    &mut std::io::Cursor::new(Vec::<u8>::new()),
                    &header,
                    &mut ctx
                )
                .is_err()
            );
            assert!(
                <WPayload as TryExtractPayloadFrom<WPayload>>::try_read(
                    &mut std::io::Cursor::new(Vec::<u8>::new()),
                    &header,
                    &mut ctx
                )
                .is_err()
            );
            assert!(
                <WPayload as ExtractPayloadFrom<WPayload>>::read(
                    &mut std::io::Cursor::new(Vec::<u8>::new()),
                    &header,
                    &mut ctx
                )
                .is_err()
            );
        }
    }
}
