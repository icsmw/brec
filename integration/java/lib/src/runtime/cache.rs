use crate::{JavaError, JavaFieldHint};
use jni::{
    Env,
    objects::{Global, JClass, JMethodID, JStaticMethodID},
    signature::RuntimeMethodSignature,
    strings::JNIString,
};
use std::sync::OnceLock;

pub(super) struct JniCache {
    pub(super) hash_map_class: Global<JClass<'static>>,
    pub(super) hash_map_ctor: JMethodID,
    pub(super) map_put: JMethodID,
    pub(super) map_get: JMethodID,
    pub(super) map_contains_key: JMethodID,
    pub(super) map_key_set: JMethodID,
    pub(super) array_list_class: Global<JClass<'static>>,
    pub(super) array_list_ctor: JMethodID,
    pub(super) list_add: JMethodID,
    pub(super) list_get: JMethodID,
    pub(super) list_size: JMethodID,
    pub(super) set_size: JMethodID,
    pub(super) set_iterator: JMethodID,
    pub(super) iterator_next: JMethodID,
    pub(super) long_class: Global<JClass<'static>>,
    pub(super) long_value_of: JStaticMethodID,
    pub(super) long_long_value: JMethodID,
    pub(super) boolean_class: Global<JClass<'static>>,
    pub(super) boolean_value_of: JStaticMethodID,
    pub(super) boolean_boolean_value: JMethodID,
    pub(super) big_integer_class: Global<JClass<'static>>,
    pub(super) big_integer_string_ctor: JMethodID,
    pub(super) object_to_string: JMethodID,
}

static JNI_CACHE: OnceLock<JniCache> = OnceLock::new();

pub(super) fn jni_cache(env: &mut Env<'_>) -> Result<&'static JniCache, JavaError> {
    if let Some(cache) = JNI_CACHE.get() {
        return Ok(cache);
    }

    let cache = init_jni_cache(env)?;
    let _ = JNI_CACHE.set(cache);
    JNI_CACHE.get().ok_or_else(|| {
        JavaError::invalid_field(JavaFieldHint::Object, "JNI cache was not initialized")
    })
}

fn init_jni_cache(env: &mut Env<'_>) -> Result<JniCache, JavaError> {
    fn class<'local>(
        env: &mut Env<'local>,
        name: &str,
        hint: JavaFieldHint,
    ) -> Result<JClass<'local>, JavaError> {
        env.find_class(JNIString::new(name))
            .map_err(|err| JavaError::invalid_field(hint, err))
    }

    fn method<'local>(
        env: &mut Env<'local>,
        class: &JClass<'local>,
        name: &str,
        sig: &str,
        hint: JavaFieldHint,
    ) -> Result<JMethodID, JavaError> {
        let sig = RuntimeMethodSignature::from_str(sig)
            .map_err(|err| JavaError::invalid_field(hint, err))?;
        env.get_method_id(class, JNIString::new(name), sig.method_signature())
            .map_err(|err| JavaError::invalid_field(hint, err))
    }

    fn static_method<'local>(
        env: &mut Env<'local>,
        class: &JClass<'local>,
        name: &str,
        sig: &str,
        hint: JavaFieldHint,
    ) -> Result<JStaticMethodID, JavaError> {
        let sig = RuntimeMethodSignature::from_str(sig)
            .map_err(|err| JavaError::invalid_field(hint, err))?;
        env.get_static_method_id(class, JNIString::new(name), sig.method_signature())
            .map_err(|err| JavaError::invalid_field(hint, err))
    }

    let hash_map_class = env
        .find_class(JNIString::new("java/util/HashMap"))
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;
    let hash_map_ctor = method(env, &hash_map_class, "<init>", "()V", JavaFieldHint::Object)?;
    let map_class = class(env, "java/util/Map", JavaFieldHint::Object)?;
    let map_put = method(
        env,
        &map_class,
        "put",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        JavaFieldHint::Object,
    )?;
    let map_get = method(
        env,
        &map_class,
        "get",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        JavaFieldHint::Object,
    )?;
    let map_contains_key = method(
        env,
        &map_class,
        "containsKey",
        "(Ljava/lang/Object;)Z",
        JavaFieldHint::Object,
    )?;
    let map_key_set = method(
        env,
        &map_class,
        "keySet",
        "()Ljava/util/Set;",
        JavaFieldHint::Object,
    )?;
    let hash_map_class = env
        .new_global_ref(hash_map_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let array_list_class = class(env, "java/util/ArrayList", JavaFieldHint::Vec)?;
    let array_list_ctor = method(env, &array_list_class, "<init>", "(I)V", JavaFieldHint::Vec)?;
    let list_class = class(env, "java/util/List", JavaFieldHint::Vec)?;
    let list_add = method(
        env,
        &list_class,
        "add",
        "(Ljava/lang/Object;)Z",
        JavaFieldHint::Vec,
    )?;
    let list_get = method(
        env,
        &list_class,
        "get",
        "(I)Ljava/lang/Object;",
        JavaFieldHint::Vec,
    )?;
    let list_size = method(env, &list_class, "size", "()I", JavaFieldHint::Vec)?;
    let array_list_class = env
        .new_global_ref(array_list_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Vec, err))?;

    let set_class = class(env, "java/util/Set", JavaFieldHint::Object)?;
    let set_size = method(env, &set_class, "size", "()I", JavaFieldHint::Object)?;
    let set_iterator = method(
        env,
        &set_class,
        "iterator",
        "()Ljava/util/Iterator;",
        JavaFieldHint::Object,
    )?;

    let iterator_class = class(env, "java/util/Iterator", JavaFieldHint::Object)?;
    let iterator_next = method(
        env,
        &iterator_class,
        "next",
        "()Ljava/lang/Object;",
        JavaFieldHint::Object,
    )?;

    let long_class = class(env, "java/lang/Long", JavaFieldHint::Object)?;
    let long_value_of = static_method(
        env,
        &long_class,
        "valueOf",
        "(J)Ljava/lang/Long;",
        JavaFieldHint::Object,
    )?;
    let long_long_value = method(env, &long_class, "longValue", "()J", JavaFieldHint::Object)?;
    let long_class = env
        .new_global_ref(long_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let boolean_class = class(env, "java/lang/Boolean", JavaFieldHint::Bool)?;
    let boolean_value_of = static_method(
        env,
        &boolean_class,
        "valueOf",
        "(Z)Ljava/lang/Boolean;",
        JavaFieldHint::Bool,
    )?;
    let boolean_boolean_value = method(
        env,
        &boolean_class,
        "booleanValue",
        "()Z",
        JavaFieldHint::Bool,
    )?;
    let boolean_class = env
        .new_global_ref(boolean_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Bool, err))?;

    let big_integer_class = class(env, "java/math/BigInteger", JavaFieldHint::Object)?;
    let big_integer_string_ctor = method(
        env,
        &big_integer_class,
        "<init>",
        "(Ljava/lang/String;)V",
        JavaFieldHint::Object,
    )?;
    let big_integer_class = env
        .new_global_ref(big_integer_class)
        .map_err(|err| JavaError::invalid_field(JavaFieldHint::Object, err))?;

    let object_class = class(env, "java/lang/Object", JavaFieldHint::Object)?;
    let object_to_string = method(
        env,
        &object_class,
        "toString",
        "()Ljava/lang/String;",
        JavaFieldHint::Object,
    )?;

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
