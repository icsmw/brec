use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[napi]
pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{
	Block::decode_napi(env, buf).map_err(|e| to_napi_error("Decode block", e))
}}

#[napi]
pub fn {}(_env: Env, val: Unknown<'_>) -> Result<Buffer> {{
	let mut buf: Vec<u8> = Vec::new();
	Block::encode_napi(val, &mut buf).map_err(|e| to_napi_error("Encode block", e))?;
	Ok(buf.into())
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[napi]
pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{
	let mut ctx = ();
	Payload::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode payload", e))
}}

#[napi]
pub fn {}(env: Env, val: Unknown<'_>) -> Result<Buffer> {{
	let mut buf: Vec<u8> = Vec::new();
	let mut ctx = ();
	Payload::encode_napi(&env, val, &mut buf, &mut ctx).map_err(|e| to_napi_error("Encode payload", e))?;
	Ok(buf.into())
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[napi]
pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{
	let mut ctx = ();
	Packet::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode packet", e))
}}

#[napi]
pub fn {}(env: Env, packet: Unknown<'_>) -> Result<Buffer> {{
	let mut buf: Vec<u8> = Vec::new();
	let mut ctx = ();
	Packet::encode_napi(&env, packet, &mut buf, &mut ctx).map_err(|e| to_napi_error("Encode packet", e))?;
	Ok(buf.into())
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}
