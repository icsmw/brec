use super::{JavaError, JavaFieldHint, map_get, map_has, map_put, new_array_list, new_hash_map};
use crate::*;
use jni::{
    JNIEnv,
    objects::{JByteArray, JObject, JString, JValue},
    sys::jboolean,
};

const PAYLOAD_FIELD_NAME: &str = "payload";
const BLOCKS_FIELD_NAME: &str = "blocks";

/// Rust <-> Java object conversion contract used by `java` helpers.
pub trait JavaObject: Sized {
    /// Converts this value into a Java object representation.
    fn to_java_object<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error>;
    /// Constructs this value from a Java object representation.
    fn from_java_object<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error>;
}

/// Schema-driven Rust <-> Java conversion used by payload nested types.
pub trait JavaConvert: Sized {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error>;
    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error>;
}

#[inline]
fn new_java_long<'local>(env: &mut JNIEnv<'local>, value: i64) -> Result<JObject<'local>, Error> {
    env.call_static_method(
        "java/lang/Long",
        "valueOf",
        "(J)Ljava/lang/Long;",
        &[JValue::Long(value)],
    )
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))
}

#[inline]
fn from_java_long<'local>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
    hint: JavaFieldHint,
) -> Result<i64, Error> {
    env.call_method(value, "longValue", "()J", &[])
        .map_err(|err| JavaError::invalid_field(hint, err))?
        .j()
        .map_err(|err| JavaError::invalid_field(hint, err))
}

#[inline]
fn new_java_bool<'local>(env: &mut JNIEnv<'local>, value: bool) -> Result<JObject<'local>, Error> {
    let raw: jboolean = if value { 1 } else { 0 };
    env.call_static_method(
        "java/lang/Boolean",
        "valueOf",
        "(Z)Ljava/lang/Boolean;",
        &[JValue::Bool(raw)],
    )
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))
}

#[inline]
fn from_java_bool<'local>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
) -> Result<bool, Error> {
    env.call_method(value, "booleanValue", "()Z", &[])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?
        .z()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))
}

#[inline]
fn new_big_integer<'local>(env: &mut JNIEnv<'local>, text: &str) -> Result<JObject<'local>, Error> {
    let jstr: JObject<'local> = env
        .new_string(text)
        .map(JObject::from)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    env.new_object(
        "java/math/BigInteger",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&jstr)],
    )
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))
}

#[inline]
fn parse_big_integer<'local, T: std::str::FromStr>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
    hint: JavaFieldHint,
) -> Result<T, Error> {
    let text_obj = env
        .call_method(value, "toString", "()Ljava/lang/String;", &[])
        .map_err(|err| JavaError::invalid_field(hint, err))?
        .l()
        .map_err(|err| JavaError::invalid_field(hint, err))?;
    let text = env
        .get_string(&JString::from(text_obj))
        .map_err(|err| JavaError::invalid_field(hint, err))?
        .to_string_lossy()
        .to_string();
    text.parse::<T>()
        .map_err(|_| JavaError::invalid_field(hint, "value is out of range"))
}

impl JavaConvert for bool {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        new_java_bool(env, *self)
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::Bool,
                "null is not allowed",
            ));
        }
        from_java_bool(env, &value)
    }
}

impl JavaConvert for String {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        env.new_string(self)
            .map(JObject::from)
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::String, err))
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::String,
                "null is not allowed",
            ));
        }
        let jstr = JString::from(value);
        Ok(env
            .get_string(&jstr)
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::String, err))?
            .into())
    }
}

macro_rules! impl_java_int_via_long {
    ($($ty:ty => $hint:expr),* $(,)?) => {
        $(
            impl JavaConvert for $ty {
                fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
                    new_java_long(env, *self as i64)
                }

                fn from_java_value<'local>(env: &mut JNIEnv<'local>, value: JObject<'local>) -> Result<Self, Error> {
                    if value.is_null() {
                        return Err(JavaError::invalid_field($hint, "null is not allowed"));
                    }
                    let raw = from_java_long(env, &value, $hint)?;
                    <$ty>::try_from(raw)
                        .map_err(|_| JavaError::invalid_field($hint, "value is out of range"))
                }
            }
        )*
    };
}

impl_java_int_via_long!(
    u8 => JavaFieldHint::U8,
    u16 => JavaFieldHint::U16,
    u32 => JavaFieldHint::U32,
    i8 => JavaFieldHint::I8,
    i16 => JavaFieldHint::I16,
    i32 => JavaFieldHint::I32,
);

impl JavaConvert for i64 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        new_big_integer(env, &self.to_string())
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::I64,
                "null is not allowed",
            ));
        }
        parse_big_integer(env, &value, JavaFieldHint::I64)
    }
}

impl JavaConvert for u64 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        new_big_integer(env, &self.to_string())
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::U64,
                "null is not allowed",
            ));
        }
        parse_big_integer(env, &value, JavaFieldHint::U64)
    }
}

impl JavaConvert for i128 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        new_big_integer(env, &self.to_string())
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::I128,
                "null is not allowed",
            ));
        }
        parse_big_integer(env, &value, JavaFieldHint::I128)
    }
}

impl JavaConvert for u128 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        new_big_integer(env, &self.to_string())
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::U128,
                "null is not allowed",
            ));
        }
        parse_big_integer(env, &value, JavaFieldHint::U128)
    }
}

impl JavaConvert for f32 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        <u32 as JavaConvert>::to_java_value(&self.to_bits(), env)
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        let bits = <u32 as JavaConvert>::from_java_value(env, value)?;
        Ok(f32::from_bits(bits))
    }
}

impl JavaConvert for f64 {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        <u64 as JavaConvert>::to_java_value(&self.to_bits(), env)
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        let bits = <u64 as JavaConvert>::from_java_value(env, value).map_err(|_| {
            JavaError::invalid_field(JavaFieldHint::F64, "expected BigInteger bits")
        })?;
        Ok(f64::from_bits(bits))
    }
}

impl<T: JavaConvert> JavaConvert for Vec<T> {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        let list = new_array_list(env, self.len() as i32)?;
        for item in self.iter() {
            let value = item.to_java_value(env)?;
            super::list_add(env, &list, &value)?;
        }
        Ok(list)
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::Vec,
                "null is not allowed",
            ));
        }
        let len = super::list_size(env, &value)?;
        let mut out = Vec::with_capacity(len as usize);
        for idx in 0..len {
            let elem = super::list_get(env, &value, idx)?;
            out.push(T::from_java_value(env, elem)?);
        }
        Ok(out)
    }
}

impl<T: JavaConvert> JavaConvert for Option<T> {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        match self {
            Some(v) => T::to_java_value(v, env),
            None => Ok(JObject::null()),
        }
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::from_java_value(env, value)?))
        }
    }
}

impl<const N: usize> JavaConvert for [u8; N] {
    fn to_java_value<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JObject<'local>, Error> {
        env.byte_array_from_slice(self)
            .map(JObject::from)
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::Blob, err))
    }

    fn from_java_value<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(JavaError::invalid_field(
                JavaFieldHint::Blob,
                "null is not allowed",
            ));
        }
        let raw = env
            .convert_byte_array(JByteArray::from(value))
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::Blob, err))?;
        raw.try_into().map_err(|bytes: Vec<u8>| {
            Error::Java(JavaError::InvalidField(
                JavaFieldHint::Blob.id().to_string(),
                format!("expected {N} bytes, got {}", bytes.len()),
            ))
        })
    }
}

impl<B: BlockDef + JavaObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + JavaObject>
    PacketDef<B, P, Inner>
{
    /// Converts packet into `Map{ blocks -> List<{}>, payload -> {} | null }`.
    pub fn to_java_object<'local>(
        &self,
        env: &mut JNIEnv<'local>,
    ) -> Result<JObject<'local>, Error> {
        let obj = new_hash_map(env)?;
        let blocks = new_array_list(env, self.blocks.len() as i32)?;
        for block in self.blocks.iter() {
            let value = block.to_java_object(env)?;
            super::list_add(env, &blocks, &value)?;
        }
        map_put(env, &obj, BLOCKS_FIELD_NAME, &blocks)?;

        let payload = match self.payload.as_ref() {
            Some(payload) => payload.to_java_object(env)?,
            None => JObject::null(),
        };
        map_put(env, &obj, PAYLOAD_FIELD_NAME, &payload)?;
        Ok(obj)
    }

    /// Parses packet from `Map{ blocks -> List<{}>, payload -> {} | null }`.
    pub fn from_java_object<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
    ) -> Result<Self, Error> {
        if value.is_null() {
            return Err(Error::Java(JavaError::InvalidObject(
                "null packet object".to_owned(),
            )));
        }
        let blocks_obj = map_get(env, &value, BLOCKS_FIELD_NAME)?;
        if blocks_obj.is_null() {
            return Err(Error::Java(JavaError::MissingField(
                BLOCKS_FIELD_NAME.to_owned(),
            )));
        }
        let blocks_len = super::list_size(env, &blocks_obj)?;
        let mut blocks = Vec::with_capacity(blocks_len as usize);
        for idx in 0..blocks_len {
            let block_obj = super::list_get(env, &blocks_obj, idx)?;
            blocks.push(B::from_java_object(env, block_obj).map_err(|err| {
                Error::Java(JavaError::InvalidField(
                    JavaFieldHint::Blocks.id().to_string(),
                    format!("index {idx}: {err}"),
                ))
            })?);
        }

        let payload = if map_has(env, &value, PAYLOAD_FIELD_NAME)? {
            let raw = map_get(env, &value, PAYLOAD_FIELD_NAME)?;
            if raw.is_null() {
                None
            } else {
                Some(Inner::from_java_object(env, raw)?)
            }
        } else {
            None
        };
        Ok(Self::new(blocks, payload))
    }

    /// Reads packet bytes and converts to Java object.
    pub fn decode_java<'local>(
        env: &mut JNIEnv<'local>,
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<JObject<'local>, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        packet.to_java_object(env)
    }

    /// Parses Java object packet and encodes into packet bytes.
    pub fn encode_java<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let mut packet = Self::from_java_object(env, value)?;
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
