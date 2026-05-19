use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("bytes_ptr: *const u8,")?;
        writer.ln("bytes_len: usize,")?;
        writer.back();
        writer.ln(") -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let bytes = match bytes_from_raw(bytes_ptr, bytes_len, \"decode block\") {")?;
        writer.tab();
        writer.ln("Ok(bytes) => bytes,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("match Block::decode_csharp(bytes) {")?;
        writer.tab();
        writer.ln("Ok(value) => into_value_handle(value),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"decode block failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("handle: *const ValueHandle,")?;
        writer.ln("out_len: *mut usize,")?;
        writer.back();
        writer.ln(") -> *mut u8 {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let handle = match value_handle_ref(handle, out_len, \"encode block\") {")?;
        writer.tab();
        writer.ln("Ok(handle) => handle,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("let mut out = Vec::new();")?;
        writer.ln("match Block::encode_csharp(handle.value.clone(), &mut out) {")?;
        writer.tab();
        writer.ln("Ok(()) => bytes_into_raw(out, out_len),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"encode block failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("bytes_ptr: *const u8,")?;
        writer.ln("bytes_len: usize,")?;
        writer.back();
        writer.ln(") -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer
            .ln("let bytes = match bytes_from_raw(bytes_ptr, bytes_len, \"decode payload\") {")?;
        writer.tab();
        writer.ln("Ok(bytes) => bytes,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("match Payload::decode_csharp(bytes, &mut ctx) {")?;
        writer.tab();
        writer.ln("Ok(value) => into_value_handle(value),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"decode payload failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("handle: *const ValueHandle,")?;
        writer.ln("out_len: *mut usize,")?;
        writer.back();
        writer.ln(") -> *mut u8 {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let handle = match value_handle_ref(handle, out_len, \"encode payload\") {")?;
        writer.tab();
        writer.ln("Ok(handle) => handle,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("let mut out = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("match Payload::encode_csharp(handle.value.clone(), &mut out, &mut ctx) {")?;
        writer.tab();
        writer.ln("Ok(()) => bytes_into_raw(out, out_len),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"encode payload failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("bytes_ptr: *const u8,")?;
        writer.ln("bytes_len: usize,")?;
        writer.back();
        writer.ln(") -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let bytes = match bytes_from_raw(bytes_ptr, bytes_len, \"decode packet\") {")?;
        writer.tab();
        writer.ln("Ok(bytes) => bytes,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("match Packet::decode_csharp(bytes, &mut ctx) {")?;
        writer.tab();
        writer.ln("Ok(value) => into_value_handle(value),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"decode packet failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(format!(
            "pub unsafe extern \"C\" fn {}(",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("handle: *const ValueHandle,")?;
        writer.ln("out_len: *mut usize,")?;
        writer.back();
        writer.ln(") -> *mut u8 {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let handle = match value_handle_ref(handle, out_len, \"encode packet\") {")?;
        writer.tab();
        writer.ln("Ok(handle) => handle,")?;
        writer.ln("Err(ptr) => return ptr,")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("let mut out = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("match Packet::encode_csharp(handle.value.clone(), &mut out, &mut ctx) {")?;
        writer.tab();
        writer.ln("Ok(()) => bytes_into_raw(out, out_len),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"encode packet failed: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}
