use jni::{
    JNIEnv,
    objects::{JByteArray, JClass, JObject},
    sys::{jbyteArray, jobject},
};
use protocol::Packet;

fn throw_runtime(env: &mut JNIEnv<'_>, message: impl AsRef<str>) {
    let _ = env.throw_new("java/lang/RuntimeException", message.as_ref());
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_ClientBindings_decodePacket<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    bytes: JByteArray<'local>,
) -> jobject {
    let bytes = match env.convert_byte_array(bytes) {
        Ok(bytes) => bytes,
        Err(err) => {
            throw_runtime(&mut env, format!("decode packet: convert input bytes failed: {err}"));
            return JObject::null().into_raw();
        }
    };

    let mut ctx = ();
    match Packet::decode_java(&mut env, &bytes, &mut ctx) {
        Ok(obj) => obj.into_raw(),
        Err(err) => {
            throw_runtime(&mut env, format!("decode packet failed: {err}"));
            JObject::null().into_raw()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_ClientBindings_encodePacket<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    packet: JObject<'local>,
) -> jbyteArray {
    let mut out = Vec::new();
    let mut ctx = ();
    if let Err(err) = Packet::encode_java(&mut env, packet, &mut out, &mut ctx) {
        throw_runtime(&mut env, format!("encode packet failed: {err}"));
        return JObject::null().into_raw() as jbyteArray;
    }

    match env.byte_array_from_slice(&out) {
        Ok(arr) => arr.into_raw(),
        Err(err) => {
            throw_runtime(&mut env, format!("encode packet: output allocation failed: {err}"));
            JObject::null().into_raw() as jbyteArray
        }
    }
}
