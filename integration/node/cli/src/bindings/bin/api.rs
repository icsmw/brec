use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer
            .ln("Block::decode_napi(env, buf).map_err(|e| to_napi_error(\"Decode block\", e))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}(_env: Env, val: Unknown<'_>) -> Result<Buffer> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln(
            "Block::encode_napi(val, &mut buf).map_err(|e| to_napi_error(\"Encode block\", e))?;",
        )?;
        writer.ln("Ok(buf.into())")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut ctx = ();")?;
        writer
            .ln("Payload::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error(\"Decode payload\", e))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}(env: Env, val: Unknown<'_>) -> Result<Buffer> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln(
            "Payload::encode_napi(&env, val, &mut buf, &mut ctx).map_err(|e| to_napi_error(\"Encode payload\", e))?;",
        )?;
        writer.ln("Ok(buf.into())")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut ctx = ();")?;
        writer.ln(
            "Packet::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error(\"Decode packet\", e))",
        )?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[napi]")?;
        writer.ln(format!(
            "pub fn {}(env: Env, packet: Unknown<'_>) -> Result<Buffer> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln(
            "Packet::encode_napi(&env, packet, &mut buf, &mut ctx).map_err(|e| to_napi_error(\"Encode packet\", e))?;",
        )?;
        writer.ln("Ok(buf.into())")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}
