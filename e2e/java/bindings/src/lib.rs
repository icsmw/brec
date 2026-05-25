use jni::{
    Env, EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::{JByteArray, JClass, JObject},
    strings::JNIString,
    sys::{jbyteArray, jobject},
};
use protocol::Packet;

fn throw_runtime(env: &mut Env<'_>, message: impl AsRef<str>) {
    let _ = env.throw_new(
        JNIString::new("java/lang/RuntimeException"),
        JNIString::new(message.as_ref()),
    );
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_ClientBindings_decodePacket<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    bytes: JByteArray<'local>,
) -> jobject {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jobject> {
            let bytes = match env.convert_byte_array(bytes) {
                Ok(bytes) => bytes,
                Err(err) => {
                    throw_runtime(
                        env,
                        format!("decode packet: convert input bytes failed: {err}"),
                    );
                    return Ok(JObject::null().into_raw());
                }
            };

            let mut ctx = ();
            match Packet::decode_java(env, &bytes, &mut ctx) {
                Ok(obj) => Ok(obj.into_raw()),
                Err(err) => {
                    throw_runtime(env, format!("decode packet failed: {err}"));
                    Ok(JObject::null().into_raw())
                }
            }
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_ClientBindings_encodePacket<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    packet: JObject<'local>,
) -> jbyteArray {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jbyteArray> {
            let mut out = Vec::new();
            let mut ctx = ();
            if let Err(err) = Packet::encode_java(env, packet, &mut out, &mut ctx) {
                throw_runtime(env, format!("encode packet failed: {err}"));
                return Ok(JObject::null().into_raw() as jbyteArray);
            }

            match env.byte_array_from_slice(&out) {
                Ok(arr) => Ok(arr.into_raw()),
                Err(err) => {
                    throw_runtime(
                        env,
                        format!("encode packet: output allocation failed: {err}"),
                    );
                    Ok(JObject::null().into_raw() as jbyteArray)
                }
            }
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}
