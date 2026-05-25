use crate::*;

fn write_decode(
    writer: &mut SourceWriter,
    jni_name: &str,
    rust_ty: &str,
    ctx: bool,
) -> Result<(), Error> {
    writer.block(format!(
        r#"
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_Client_{jni_name}<'local>(
	mut unowned_env: EnvUnowned<'local>,
	_class: JClass<'local>,
	bytes: JByteArray<'local>,
) -> jobject {{
	unowned_env.with_env(|env| -> jni::errors::Result<jobject> {{
	let bytes = match env.convert_byte_array(bytes) {{
		Ok(bytes) => bytes,
		Err(err) => {{
			throw_runtime(env, format!("{jni_name}: convert input bytes failed: {{err}}"));
			return Ok(JObject::null().into_raw());
		}}
	}};
"#
    ))?;
    if ctx {
        writer.ln("\tlet mut ctx = ();")?;
        writer.ln(format!(
            "\tmatch {rust_ty}::decode_java(env, &bytes, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!("\tmatch {rust_ty}::decode_java(env, &bytes) {{"))?;
    }
    writer.block(format!(
        r#"
	Ok(obj) => Ok(obj.into_raw()),
	Err(err) => {{
		throw_runtime(env, format!("{jni_name} failed: {{err}}"));
		Ok(JObject::null().into_raw())
	}}
}}
}}).resolve::<ThrowRuntimeExAndDefault>()
}}
"#
    ))
}

fn write_encode(
    writer: &mut SourceWriter,
    jni_name: &str,
    rust_ty: &str,
    arg_name: &str,
    ctx: bool,
) -> Result<(), Error> {
    writer.block(format!(
        r#"
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_icsmw_brec_Client_{jni_name}<'local>(
	mut unowned_env: EnvUnowned<'local>,
	_class: JClass<'local>,
	{arg_name}: JObject<'local>,
) -> jbyteArray {{
	unowned_env.with_env(|env| -> jni::errors::Result<jbyteArray> {{
	let mut out = Vec::new();
"#
    ))?;
    if ctx {
        writer.ln("\tlet mut ctx = ();")?;
        writer.ln(format!(
            "\tif let Err(err) = {rust_ty}::encode_java(env, {arg_name}, &mut out, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!(
            "\tif let Err(err) = {rust_ty}::encode_java(env, {arg_name}, &mut out) {{"
        ))?;
    }
    writer.block(format!(
        r#"
	throw_runtime(env, format!("{jni_name} failed: {{err}}"));
	return Ok(JObject::null().into_raw() as jbyteArray);
}}

match env.byte_array_from_slice(&out) {{
	Ok(arr) => Ok(arr.into_raw()),
	Err(err) => {{
		throw_runtime(env, format!("{jni_name}: output allocation failed: {{err}}"));
		Ok(JObject::null().into_raw() as jbyteArray)
	}}
}}
}}).resolve::<ThrowRuntimeExAndDefault>()
}}
"#
    ))
}

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_decode(writer, "decodeBlockNative", "Block", false)?;
        write_encode(writer, "encodeBlockNative", "Block", "block", false)
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_decode(writer, "decodePayloadNative", "Payload", true)?;
        write_encode(writer, "encodePayloadNative", "Payload", "payload", true)
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_decode(writer, "decodePacketNative", "Packet", true)?;
        write_encode(writer, "encodePacketNative", "Packet", "packet", true)
    }
}
