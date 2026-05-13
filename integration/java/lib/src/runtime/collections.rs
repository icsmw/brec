use super::cache::jni_cache;
use crate::{JavaError, JavaFieldHint};
use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
    signature::{Primitive, ReturnType},
};

#[inline]
pub fn new_hash_map<'local>(env: &mut JNIEnv<'local>) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    // SAFETY: method id is resolved once from java/util/HashMap.<init>()V and
    // kept alive by a global class reference in `JniCache`.
    unsafe { env.new_object_unchecked(&cache.hash_map_class, cache.hash_map_ctor, &[]) }
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
    let cache = jni_cache(env)?;
    let args = [
        JValue::Object(&key_obj).as_jni(),
        JValue::Object(value).as_jni(),
    ];
    // SAFETY: `map_put` is resolved for Map.put(Object,Object):Object.
    unsafe { env.call_method_unchecked(map, cache.map_put, ReturnType::Object, &args) }
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
    let cache = jni_cache(env)?;
    let args = [JValue::Object(&key_obj).as_jni()];
    let raw = unsafe { env.call_method_unchecked(map, cache.map_get, ReturnType::Object, &args) }
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
    let cache = jni_cache(env)?;
    let args = [JValue::Object(&key_obj).as_jni()];
    // SAFETY: `map_contains_key` is resolved for Map.containsKey(Object):boolean.
    unsafe {
        env.call_method_unchecked(
            map,
            cache.map_contains_key,
            ReturnType::Primitive(Primitive::Boolean),
            &args,
        )
    }
    .map_err(|err| JavaError::invalid_field_name(key, err))?
    .z()
    .map_err(|err| JavaError::invalid_field_name(key, err))
}

#[inline]
pub fn map_keys_len_and_first<'local>(
    env: &mut JNIEnv<'local>,
    map: &JObject<'local>,
) -> Result<(i32, Option<String>), JavaError> {
    let cache = jni_cache(env)?;
    let keys_set =
        unsafe { env.call_method_unchecked(map, cache.map_key_set, ReturnType::Object, &[]) }
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
            .l()
            .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let len = unsafe {
        env.call_method_unchecked(
            &keys_set,
            cache.set_size,
            ReturnType::Primitive(Primitive::Int),
            &[],
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
    .i()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    if len == 0 {
        return Ok((0, None));
    }

    let iterator = unsafe {
        env.call_method_unchecked(&keys_set, cache.set_iterator, ReturnType::Object, &[])
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let first = unsafe {
        env.call_method_unchecked(&iterator, cache.iterator_next, ReturnType::Object, &[])
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
    .l()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let rust = env
        .get_string(&JString::from(first))
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?
        .into();
    Ok((len, Some(rust)))
}

#[inline]
pub fn new_array_list<'local>(
    env: &mut JNIEnv<'local>,
    cap: i32,
) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    let args = [JValue::Int(cap).as_jni()];
    // SAFETY: method id is resolved once from java/util/ArrayList.<init>(I)V
    // and kept alive by a global class reference in `JniCache`.
    unsafe { env.new_object_unchecked(&cache.array_list_class, cache.array_list_ctor, &args) }
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}

#[inline]
pub fn list_add<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
    value: &JObject<'local>,
) -> Result<(), JavaError> {
    let cache = jni_cache(env)?;
    let args = [JValue::Object(value).as_jni()];
    // SAFETY: `list_add` is resolved for List.add(Object):boolean.
    unsafe {
        env.call_method_unchecked(
            list,
            cache.list_add,
            ReturnType::Primitive(Primitive::Boolean),
            &args,
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    Ok(())
}

#[inline]
pub fn list_get<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
    idx: i32,
) -> Result<JObject<'local>, JavaError> {
    let cache = jni_cache(env)?;
    let args = [JValue::Int(idx).as_jni()];
    // SAFETY: `list_get` is resolved for List.get(int):Object.
    unsafe { env.call_method_unchecked(list, cache.list_get, ReturnType::Object, &args) }
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?
        .l()
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}

#[inline]
pub fn list_size<'local>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
) -> Result<i32, JavaError> {
    let cache = jni_cache(env)?;
    // SAFETY: `list_size` is resolved for List.size():int.
    unsafe {
        env.call_method_unchecked(
            list,
            cache.list_size,
            ReturnType::Primitive(Primitive::Int),
            &[],
        )
    }
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?
    .i()
    .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))
}
