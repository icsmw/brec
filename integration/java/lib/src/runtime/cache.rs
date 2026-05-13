use crate::{JavaError, JavaFieldHint};
use jni::{
    JNIEnv,
    objects::{GlobalRef, JMethodID, JStaticMethodID},
};
use std::sync::OnceLock;

pub(super) struct JniCache {
    pub(super) hash_map_class: GlobalRef,
    pub(super) hash_map_ctor: JMethodID,
    pub(super) map_put: JMethodID,
    pub(super) map_get: JMethodID,
    pub(super) map_contains_key: JMethodID,
    pub(super) map_key_set: JMethodID,
    pub(super) array_list_class: GlobalRef,
    pub(super) array_list_ctor: JMethodID,
    pub(super) list_add: JMethodID,
    pub(super) list_get: JMethodID,
    pub(super) list_size: JMethodID,
    pub(super) set_size: JMethodID,
    pub(super) set_iterator: JMethodID,
    pub(super) iterator_next: JMethodID,
    pub(super) long_class: GlobalRef,
    pub(super) long_value_of: JStaticMethodID,
    pub(super) long_long_value: JMethodID,
    pub(super) boolean_class: GlobalRef,
    pub(super) boolean_value_of: JStaticMethodID,
    pub(super) boolean_boolean_value: JMethodID,
    pub(super) big_integer_class: GlobalRef,
    pub(super) big_integer_string_ctor: JMethodID,
    pub(super) object_to_string: JMethodID,
}

static JNI_CACHE: OnceLock<JniCache> = OnceLock::new();

pub(super) fn jni_cache(env: &mut JNIEnv<'_>) -> Result<&'static JniCache, JavaError> {
    if let Some(cache) = JNI_CACHE.get() {
        return Ok(cache);
    }

    let cache = init_jni_cache(env)?;
    let _ = JNI_CACHE.set(cache);
    JNI_CACHE.get().ok_or_else(|| {
        JavaError::invalid_field(JavaFieldHint::Object, "JNI cache was not initialized")
    })
}

fn init_jni_cache(env: &mut JNIEnv<'_>) -> Result<JniCache, JavaError> {
    let hash_map_class = env
        .find_class("java/util/HashMap")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let hash_map_ctor = env
        .get_method_id(&hash_map_class, "<init>", "()V")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let map_class = env
        .find_class("java/util/Map")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let map_put = env
        .get_method_id(
            &map_class,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        )
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let map_get = env
        .get_method_id(&map_class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let map_contains_key = env
        .get_method_id(&map_class, "containsKey", "(Ljava/lang/Object;)Z")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let map_key_set = env
        .get_method_id(&map_class, "keySet", "()Ljava/util/Set;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let hash_map_class = env
        .new_global_ref(hash_map_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let array_list_class = env
        .find_class("java/util/ArrayList")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let array_list_ctor = env
        .get_method_id(&array_list_class, "<init>", "(I)V")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let list_class = env
        .find_class("java/util/List")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let list_add = env
        .get_method_id(&list_class, "add", "(Ljava/lang/Object;)Z")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let list_get = env
        .get_method_id(&list_class, "get", "(I)Ljava/lang/Object;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let list_size = env
        .get_method_id(&list_class, "size", "()I")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;
    let array_list_class = env
        .new_global_ref(array_list_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;

    let set_class = env
        .find_class("java/util/Set")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let set_size = env
        .get_method_id(&set_class, "size", "()I")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let set_iterator = env
        .get_method_id(&set_class, "iterator", "()Ljava/util/Iterator;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let iterator_class = env
        .find_class("java/util/Iterator")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let iterator_next = env
        .get_method_id(&iterator_class, "next", "()Ljava/lang/Object;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let long_class = env
        .find_class("java/lang/Long")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let long_value_of = env
        .get_static_method_id(&long_class, "valueOf", "(J)Ljava/lang/Long;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let long_long_value = env
        .get_method_id(&long_class, "longValue", "()J")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let long_class = env
        .new_global_ref(long_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let boolean_class = env
        .find_class("java/lang/Boolean")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?;
    let boolean_value_of = env
        .get_static_method_id(&boolean_class, "valueOf", "(Z)Ljava/lang/Boolean;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?;
    let boolean_boolean_value = env
        .get_method_id(&boolean_class, "booleanValue", "()Z")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?;
    let boolean_class = env
        .new_global_ref(boolean_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?;

    let big_integer_class = env
        .find_class("java/math/BigInteger")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let big_integer_string_ctor = env
        .get_method_id(&big_integer_class, "<init>", "(Ljava/lang/String;)V")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let big_integer_class = env
        .new_global_ref(big_integer_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let object_class = env
        .find_class("java/lang/Object")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let object_to_string = env
        .get_method_id(&object_class, "toString", "()Ljava/lang/String;")
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    Ok(JniCache {
        hash_map_class,
        hash_map_ctor,
        map_put,
        map_get,
        map_contains_key,
        map_key_set,
        array_list_class,
        array_list_ctor,
        list_add,
        list_get,
        list_size,
        set_size,
        set_iterator,
        iterator_next,
        long_class,
        long_value_of,
        long_long_value,
        boolean_class,
        boolean_value_of,
        boolean_boolean_value,
        big_integer_class,
        big_integer_string_ctor,
        object_to_string,
    })
}
