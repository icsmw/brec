mod error;
mod packet;

pub use error::*;
pub use jni;
use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
};
pub use packet::*;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Clone, Copy, Debug)]
pub enum JavaFieldHint {
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
pub fn new_hash_map<'local>(env: &mut JNIEnv<'local>) -> Result<JObject<'local>, JavaError> {
    env.new_object("java/util/HashMap", "()V", &[])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))
}

#[inline]
pub fn map_put<'local>(
    env: &mut JNIEnv<'local>,
    map: &JObject<'local>,
    key: &str,
    value: &JObject<'local>,
) -> Result<(), JavaError> {
    let key_obj: JObject<'local> = env
        .new_string(key)
        .map(JObject::from)
        .map_err(|err| JavaError::invalid_field_name(key, err))?;
    env.call_method(
        map,
        "put",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[JValue::Object(&key_obj), JValue::Object(value)],
    )
    .map_err(|err| JavaError::invalid_field_name(key, err))?;
    Ok(())
}

#[inline]
pub fn map_get<'local>(
    env: &mut JNIEnv<'local>,
    map: &JObject<'local>,
    key: &str,
) -> Result<JObject<'local>, JavaError> {
    let key_obj: JObject<'local> = env
        .new_string(key)
        .map(JObject::from)
        .map_err(|err| JavaError::invalid_field_name(key, err))?;
    let raw = env
        .call_method(
            map,
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&key_obj)],
        )
        .map_err(|err| JavaError::invalid_field_name(key, err))?
        .l()
        .map_err(|err| JavaError::invalid_field_name(key, err))?;
    Ok(raw)
}

#[inline]
pub fn map_has<'local>(
    env: &mut JNIEnv<'local>,
    map: &JObject<'local>,
    key: &str,
) -> Result<bool, JavaError> {
    let key_obj: JObject<'local> = env
        .new_string(key)
        .map(JObject::from)
        .map_err(|err| JavaError::invalid_field_name(key, err))?;
    env.call_method(
        map,
        "containsKey",
        "(Ljava/lang/Object;)Z",
        &[JValue::Object(&key_obj)],
    )
    .map_err(|err| JavaError::invalid_field_name(key, err))?
    .z()
    .map_err(|err| JavaError::invalid_field_name(key, err))
}

#[inline]
pub fn map_keys_len_and_first<'local>(
    env: &mut JNIEnv<'local>,
    map: &JObject<'local>,
) -> Result<(i32, Option<String>), JavaError> {
    let keys_set = env
        .call_method(map, "keySet", "()Ljava/util/Set;", &[])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
        .l()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let keys_array = env
        .call_method(&keys_set, "toArray", "()[Ljava/lang/Object;", &[])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
        .l()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let arr = jni::objects::JObjectArray::from(keys_array);
    let len = env
        .get_array_length(&arr)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    if len == 0 {
        return Ok((0, None));
    }
    let first = env
        .get_object_array_element(&arr, 0)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let jstr = JString::from(first);
    let rust = env
        .get_string(&jstr)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
        .into();
    Ok((len, Some(rust)))
}

#[inline]
pub fn new_array_list<'local>(
    env: &mut JNIEnv<'local>,
    cap: i32,
) -> Result<JObject<'local>, JavaError> {
    env.new_object("java/util/ArrayList", "(I)V", &[JValue::Int(cap)])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}

#[inline]
pub fn list_add<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
    value: &JObject<'local>,
) -> Result<(), JavaError> {
    env.call_method(
        list,
        "add",
        "(Ljava/lang/Object;)Z",
        &[JValue::Object(value)],
    )
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    Ok(())
}

#[inline]
pub fn list_get<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
    idx: i32,
) -> Result<JObject<'local>, JavaError> {
    env.call_method(list, "get", "(I)Ljava/lang/Object;", &[JValue::Int(idx)])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?
        .l()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}

#[inline]
pub fn list_size<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
) -> Result<i32, JavaError> {
    env.call_method(list, "size", "()I", &[])
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?
        .i()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}
