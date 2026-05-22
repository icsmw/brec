use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	bytes_ptr: *const u8,
	bytes_len: usize,
) -> *mut ValueHandle {{
	clear_last_error();
	let bytes = match bytes_from_raw(bytes_ptr, bytes_len, "decode block") {{
		Ok(bytes) => bytes,
		Err(ptr) => return ptr,
	}};
	match Block::decode_csharp(bytes) {{
		Ok(value) => into_value_handle(value),
		Err(err) => fail_ptr(format!("decode block failed: {{err}}")),
	}}
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	handle: *const ValueHandle,
	out_len: *mut usize,
) -> *mut u8 {{
	clear_last_error();
	let handle = match value_handle_ref(handle, out_len, "encode block") {{
		Ok(handle) => handle,
		Err(ptr) => return ptr,
	}};
	let mut out = Vec::new();
	match Block::encode_csharp(handle.value.clone(), &mut out) {{
		Ok(()) => bytes_into_raw(out, out_len),
		Err(err) => fail_ptr(format!("encode block failed: {{err}}")),
	}}
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	bytes_ptr: *const u8,
	bytes_len: usize,
) -> *mut ValueHandle {{
	clear_last_error();
	let bytes = match bytes_from_raw(bytes_ptr, bytes_len, "decode payload") {{
		Ok(bytes) => bytes,
		Err(ptr) => return ptr,
	}};
	let mut ctx = ();
	match Payload::decode_csharp(bytes, &mut ctx) {{
		Ok(value) => into_value_handle(value),
		Err(err) => fail_ptr(format!("decode payload failed: {{err}}")),
	}}
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	handle: *const ValueHandle,
	out_len: *mut usize,
) -> *mut u8 {{
	clear_last_error();
	let handle = match value_handle_ref(handle, out_len, "encode payload") {{
		Ok(handle) => handle,
		Err(ptr) => return ptr,
	}};
	let mut out = Vec::new();
	let mut ctx = ();
	match Payload::encode_csharp(handle.value.clone(), &mut out, &mut ctx) {{
		Ok(()) => bytes_into_raw(out, out_len),
		Err(err) => fail_ptr(format!("encode payload failed: {{err}}")),
	}}
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	bytes_ptr: *const u8,
	bytes_len: usize,
) -> *mut ValueHandle {{
	clear_last_error();
	let bytes = match bytes_from_raw(bytes_ptr, bytes_len, "decode packet") {{
		Ok(bytes) => bytes,
		Err(ptr) => return ptr,
	}};
	let mut ctx = ();
	match Packet::decode_csharp(bytes, &mut ctx) {{
		Ok(value) => into_value_handle(value),
		Err(err) => fail_ptr(format!("decode packet failed: {{err}}")),
	}}
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn {}(
	handle: *const ValueHandle,
	out_len: *mut usize,
) -> *mut u8 {{
	clear_last_error();
	let handle = match value_handle_ref(handle, out_len, "encode packet") {{
		Ok(handle) => handle,
		Err(ptr) => return ptr,
	}};
	let mut out = Vec::new();
	let mut ctx = ();
	match Packet::encode_csharp(handle.value.clone(), &mut out, &mut ctx) {{
		Ok(()) => bytes_into_raw(out, out_len),
		Err(err) => fail_ptr(format!("encode packet failed: {{err}}")),
	}}
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}
