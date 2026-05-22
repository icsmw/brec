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
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	bytes: JByteArray<'local>,
) -> jobject {{
	let bytes = match env.convert_byte_array(bytes) {{
		Ok(bytes) => bytes,
		Err(err) => {{
			throw_runtime(&mut env, format!("{jni_name}: convert input bytes failed: {{err}}"));
			return JObject::null().into_raw();
		}}
	}};
"#
    ))?;
    if ctx {
        writer.ln("\tlet mut ctx = ();")?;
        writer.ln(format!(
            "\tmatch {rust_ty}::decode_java(&mut env, &bytes, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!(
            "\tmatch {rust_ty}::decode_java(&mut env, &bytes) {{"
        ))?;
    }
    writer.block(format!(
        r#"
	Ok(obj) => obj.into_raw(),
	Err(err) => {{
		throw_runtime(&mut env, format!("{jni_name} failed: {{err}}"));
		JObject::null().into_raw()
	}}
}}
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
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	{arg_name}: JObject<'local>,
) -> jbyteArray {{
	let mut out = Vec::new();
"#
    ))?;
    if ctx {
        writer.ln("\tlet mut ctx = ();")?;
        writer.ln(format!(
            "\tif let Err(err) = {rust_ty}::encode_java(&mut env, {arg_name}, &mut out, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!(
            "\tif let Err(err) = {rust_ty}::encode_java(&mut env, {arg_name}, &mut out) {{"
        ))?;
    }
    writer.block(format!(
        r#"
	throw_runtime(&mut env, format!("{jni_name} failed: {{err}}"));
	return JObject::null().into_raw() as jbyteArray;
}}

match env.byte_array_from_slice(&out) {{
	Ok(arr) => arr.into_raw(),
	Err(err) => {{
		throw_runtime(&mut env, format!("{jni_name}: output allocation failed: {{err}}"));
		JObject::null().into_raw() as jbyteArray
	}}
}}
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
