use super::cache::jni_cache;
use crate::{JavaError, JavaFieldHint};
use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
    signature::{Primitive, ReturnType},
};

pub(crate) fn new_java_long<'local>(
    env: &mut JNIEnv<'local>,
    value: i64,
) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    let args = [JValue::Long(value).as_jni()];
    // SAFETY: `long_value_of` is resolved for Long.valueOf(long):Long.
    unsafe {
        env.call_static_method_unchecked(
            &cache.long_class,
            cache.long_value_of,
            ReturnType::Object,
            &args,
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))
}

pub(crate) fn java_long_value<'local>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
    hint: JavaFieldHint,
) -> Result<i64, JavaError> {
    let cache = jni_cache(env)?;
    // SAFETY: `long_long_value` is resolved for Long.longValue():long.
    unsafe {
        env.call_method_unchecked(
            value,
            cache.long_long_value,
            ReturnType::Primitive(Primitive::Long),
            &[],
        )
    }
    .map_err(|err| JavaError::invalid_field(hint, err))?
    .j()
    .map_err(|err| JavaError::invalid_field(hint, err))
}

pub(crate) fn new_java_bool<'local>(
    env: &mut JNIEnv<'local>,
    value: bool,
) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    let raw = if value { 1 } else { 0 };
    let args = [JValue::Bool(raw).as_jni()];
    // SAFETY: `boolean_value_of` is resolved for Boolean.valueOf(boolean):Boolean.
    unsafe {
        env.call_static_method_unchecked(
            &cache.boolean_class,
            cache.boolean_value_of,
            ReturnType::Object,
            &args,
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))
}

pub(crate) fn java_bool_value<'local>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
) -> Result<bool, JavaError> {
    let cache = jni_cache(env)?;
    // SAFETY: `boolean_boolean_value` is resolved for Boolean.booleanValue():boolean.
    unsafe {
        env.call_method_unchecked(
            value,
            cache.boolean_boolean_value,
            ReturnType::Primitive(Primitive::Boolean),
            &[],
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?
    .z()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))
}

pub(crate) fn new_big_integer<'local>(
    env: &mut JNIEnv<'local>,
    text: &str,
) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    let jstr: JObject<'local> = env
        .new_string(text)
        .map(JObject::from)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let args = [JValue::Object(&jstr).as_jni()];
    // SAFETY: `big_integer_string_ctor` is resolved for BigInteger(String).
    unsafe {
        env.new_object_unchecked(
            &cache.big_integer_class,
            cache.big_integer_string_ctor,
            &args,
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))
}

pub(crate) fn java_to_string<'local>(
    env: &mut JNIEnv<'local>,
    value: &JObject<'local>,
    hint: JavaFieldHint,
) -> Result<String, JavaError> {
    let cache = jni_cache(env)?;
    // SAFETY: `object_to_string` is resolved for Object.toString():String and
    // is valid for every non-null Java object.
    let text_obj = unsafe {
        env.call_method_unchecked(value, cache.object_to_string, ReturnType::Object, &[])
    }
    .map_err(|err| JavaError::invalid_field(hint, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(hint, err))?;
    Ok(env
        .get_string(&JString::from(text_obj))
        .map_err(|err| JavaError::invalid_field(hint, err))?
        .to_string_lossy()
        .to_string())
}
