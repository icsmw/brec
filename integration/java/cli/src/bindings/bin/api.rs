use crate::*;

fn write_decode(
    writer: &mut SourceWriter,
    jni_name: &str,
    rust_ty: &str,
    ctx: bool,
) -> Result<(), Error> {
    writer.ln("#[unsafe(no_mangle)]")?;
    writer.ln(format!(
        "pub extern \"system\" fn Java_com_icsmw_brec_Client_{jni_name}<'local>("
    ))?;
    writer.tab();
    writer.ln("mut env: JNIEnv<'local>,")?;
    writer.ln("_class: JClass<'local>,")?;
    writer.ln("bytes: JByteArray<'local>,")?;
    writer.back();
    writer.ln(") -> jobject {")?;
    writer.tab();
    writer.ln("let bytes = match env.convert_byte_array(bytes) {")?;
    writer.tab();
    writer.ln("Ok(bytes) => bytes,")?;
    writer.ln("Err(err) => {")?;
    writer.tab();
    writer.ln(format!(
        "throw_runtime(&mut env, format!(\"{jni_name}: convert input bytes failed: {{err}}\"));"
    ))?;
    writer.ln("return JObject::null().into_raw();")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("};")?;
    if ctx {
        writer.ln("let mut ctx = ();")?;
        writer.ln(format!(
            "match {rust_ty}::decode_java(&mut env, &bytes, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!("match {rust_ty}::decode_java(&mut env, &bytes) {{"))?;
    }
    writer.tab();
    writer.ln("Ok(obj) => obj.into_raw(),")?;
    writer.ln("Err(err) => {")?;
    writer.tab();
    writer.ln(format!(
        "throw_runtime(&mut env, format!(\"{jni_name} failed: {{err}}\"));"
    ))?;
    writer.ln("JObject::null().into_raw()")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")
}

fn write_encode(
    writer: &mut SourceWriter,
    jni_name: &str,
    rust_ty: &str,
    arg_name: &str,
    ctx: bool,
) -> Result<(), Error> {
    writer.ln("#[unsafe(no_mangle)]")?;
    writer.ln(format!(
        "pub extern \"system\" fn Java_com_icsmw_brec_Client_{jni_name}<'local>("
    ))?;
    writer.tab();
    writer.ln("mut env: JNIEnv<'local>,")?;
    writer.ln("_class: JClass<'local>,")?;
    writer.ln(format!("{arg_name}: JObject<'local>,"))?;
    writer.back();
    writer.ln(") -> jbyteArray {")?;
    writer.tab();
    writer.ln("let mut out = Vec::new();")?;
    if ctx {
        writer.ln("let mut ctx = ();")?;
        writer.ln(format!(
            "if let Err(err) = {rust_ty}::encode_java(&mut env, {arg_name}, &mut out, &mut ctx) {{"
        ))?;
    } else {
        writer.ln(format!(
            "if let Err(err) = {rust_ty}::encode_java(&mut env, {arg_name}, &mut out) {{"
        ))?;
    }
    writer.tab();
    writer.ln(format!(
        "throw_runtime(&mut env, format!(\"{jni_name} failed: {{err}}\"));"
    ))?;
    writer.ln("return JObject::null().into_raw() as jbyteArray;")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("match env.byte_array_from_slice(&out) {")?;
    writer.tab();
    writer.ln("Ok(arr) => arr.into_raw(),")?;
    writer.ln("Err(err) => {")?;
    writer.tab();
    writer.ln(format!(
        "throw_runtime(&mut env, format!(\"{jni_name}: output allocation failed: {{err}}\"));"
    ))?;
    writer.ln("JObject::null().into_raw() as jbyteArray")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")
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
