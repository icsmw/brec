use crate::*;

/// Marker for the generated Rust `src/lib.rs` binding file.
pub struct BindingsLibFile;

impl FileName for BindingsLibFile {
    const FILE_NAME: &'static str = "lib.rs";
}

impl<'a> SourceWritable for ApiFile<'a, BindingsLibFile> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_rust(writer)?;
        Ok(())
    }
}

impl<'a> RustWritable for ApiFile<'a, BindingsLibFile> {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(self.file_name(), self.model).write(writer)?;
        writer.ln("use protocol::{Block, Packet, Payload};")?;
        writer.ln("use std::cell::RefCell;")?;
        writer.ln("use std::ffi::{CString, c_char};")?;
        writer.ln("use std::collections::BTreeMap;")?;
        writer.ln("")?;
        writer.ln("pub struct ValueHandle {")?;
        writer.tab();
        writer.ln("value: brec::csharp_feat::CSharpValue,")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("thread_local! {")?;
        writer.tab();
        writer.ln("static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn sanitize_nul_bytes(input: impl Into<String>) -> String {")?;
        writer.tab();
        writer.ln("input.into().replace('\\0', \" \")")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn set_last_error(message: impl Into<String>) {")?;
        writer.tab();
        writer.ln("let text = sanitize_nul_bytes(message);")?;
        writer.ln("LAST_ERROR.with(|cell| {")?;
        writer.tab();
        writer.ln("*cell.borrow_mut() = CString::new(text).ok();")?;
        writer.back();
        writer.ln("});")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn clear_last_error() {")?;
        writer.tab();
        writer.ln("LAST_ERROR.with(|cell| {")?;
        writer.tab();
        writer.ln("*cell.borrow_mut() = None;")?;
        writer.back();
        writer.ln("});")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn fail_ptr<T>(message: impl Into<String>) -> *mut T {")?;
        writer.tab();
        writer.ln("set_last_error(message);")?;
        writer.ln("std::ptr::null_mut()")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln(
            "fn into_value_handle(value: brec::csharp_feat::CSharpValue) -> *mut ValueHandle {",
        )?;
        writer.tab();
        writer.ln("Box::into_raw(Box::new(ValueHandle { value }))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn bytes_into_raw(mut bytes: Vec<u8>, out_len: *mut usize) -> *mut u8 {")?;
        writer.tab();
        writer.ln("if out_len.is_null() {")?;
        writer.tab();
        writer.ln("return fail_ptr(\"output length pointer is null\");")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("let ptr = bytes.as_mut_ptr();")?;
        writer.ln("let len = bytes.len();")?;
        writer.ln("unsafe { *out_len = len; }")?;
        writer.ln("std::mem::forget(bytes);")?;
        writer.ln("ptr")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn bytes_from_raw<'a>(")?;
        writer.tab();
        writer.ln("bytes_ptr: *const u8,")?;
        writer.ln("bytes_len: usize,")?;
        writer.ln("op: &str,")?;
        writer.back();
        writer.ln(") -> Result<&'a [u8], *mut ValueHandle> {")?;
        writer.tab();
        writer.ln("if bytes_ptr.is_null() && bytes_len != 0 {")?;
        writer.tab();
        writer.ln("return Err(fail_ptr(format!(\"{op}: input pointer is null\")));")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("if bytes_len == 0 {")?;
        writer.tab();
        writer.ln("return Ok(&[]);")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("Ok(unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) })")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn value_handle_ref<'a>(")?;
        writer.tab();
        writer.ln("handle: *const ValueHandle,")?;
        writer.ln("_out_len: *mut usize,")?;
        writer.ln("op: &str,")?;
        writer.back();
        writer.ln(") -> Result<&'a ValueHandle, *mut u8> {")?;
        writer.tab();
        writer.ln("if handle.is_null() {")?;
        writer.tab();
        writer.ln("set_last_error(format!(\"{op}: handle is null\"));")?;
        writer.ln("return Err(std::ptr::null_mut());")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("Ok(unsafe { &*handle })")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub extern \"C\" fn bindings_last_error_message() -> *const c_char {")?;
        writer.tab();
        writer.ln("LAST_ERROR.with(|cell| {")?;
        writer.tab();
        writer.ln("if let Some(msg) = cell.borrow().as_ref() {")?;
        writer.tab();
        writer.ln("msg.as_ptr()")?;
        writer.back();
        writer.ln("} else {")?;
        writer.tab();
        writer.ln("std::ptr::null()")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("})")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_free(handle: *mut ValueHandle) {")?;
        writer.tab();
        writer.ln("if handle.is_null() {")?;
        writer.tab();
        writer.ln("return;")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("unsafe { let _ = Box::from_raw(handle); }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_bytes_free(ptr: *mut u8, len: usize) {")?;
        writer.tab();
        writer.ln("if ptr.is_null() {")?;
        writer.tab();
        writer.ln("return;")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("unsafe {")?;
        writer.tab();
        writer.ln("let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, len);")?;
        writer.ln("let _ = Box::<[u8]>::from_raw(slice_ptr);")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn value_ref<'a>(handle: *const ValueHandle, op: &str) -> Result<&'a brec::csharp_feat::CSharpValue, ()> {")?;
        writer.tab();
        writer.ln("if handle.is_null() {")?;
        writer.tab();
        writer.ln("set_last_error(format!(\"{op}: handle is null\"));")?;
        writer.ln("return Err(());")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("Ok(unsafe { &(*handle).value })")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("fn value_mut<'a>(handle: *mut ValueHandle, op: &str) -> Result<&'a mut brec::csharp_feat::CSharpValue, ()> {")?;
        writer.tab();
        writer.ln("if handle.is_null() {")?;
        writer.tab();
        writer.ln("set_last_error(format!(\"{op}: handle is null\"));")?;
        writer.ln("return Err(());")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("Ok(unsafe { &mut (*handle).value })")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(
            "pub unsafe extern \"C\" fn bindings_value_kind(handle: *const ValueHandle) -> i32 {",
        )?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let Ok(value) = value_ref(handle, \"value kind\") else { return -1; };")?;
        writer.ln("match value {")?;
        writer.tab();
        writer.ln("brec::csharp_feat::CSharpValue::Null => 0,")?;
        writer.ln("brec::csharp_feat::CSharpValue::Bool(_) => 1,")?;
        writer.ln("brec::csharp_feat::CSharpValue::U8(_) => 2,")?;
        writer.ln("brec::csharp_feat::CSharpValue::U16(_) => 3,")?;
        writer.ln("brec::csharp_feat::CSharpValue::U32(_) => 4,")?;
        writer.ln("brec::csharp_feat::CSharpValue::U64(_) => 5,")?;
        writer.ln("brec::csharp_feat::CSharpValue::U128(_) => 6,")?;
        writer.ln("brec::csharp_feat::CSharpValue::I8(_) => 7,")?;
        writer.ln("brec::csharp_feat::CSharpValue::I16(_) => 8,")?;
        writer.ln("brec::csharp_feat::CSharpValue::I32(_) => 9,")?;
        writer.ln("brec::csharp_feat::CSharpValue::I64(_) => 10,")?;
        writer.ln("brec::csharp_feat::CSharpValue::I128(_) => 11,")?;
        writer.ln("brec::csharp_feat::CSharpValue::F32Bits(_) => 12,")?;
        writer.ln("brec::csharp_feat::CSharpValue::F64Bits(_) => 13,")?;
        writer.ln("brec::csharp_feat::CSharpValue::String(_) => 14,")?;
        writer.ln("brec::csharp_feat::CSharpValue::Bytes(_) => 15,")?;
        writer.ln("brec::csharp_feat::CSharpValue::Array(_) => 16,")?;
        writer.ln("brec::csharp_feat::CSharpValue::Object(_) => 17,")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for line in [
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_null() -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Null) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_bool(value: bool) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Bool(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_u8(value: u8) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U8(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_u16(value: u16) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U16(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_u32(value: u32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U32(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_u64(value: u64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U64(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_i8(value: i8) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I8(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_i16(value: i16) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I16(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_i32(value: i32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I32(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_i64(value: i64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I64(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_f32_bits(value: u32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::F32Bits(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_f64_bits(value: u64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::F64Bits(value)) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_array(capacity: usize) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Array(Vec::with_capacity(capacity))) }",
            "#[unsafe(no_mangle)]",
            "pub extern \"C\" fn bindings_value_object() -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Object(BTreeMap::new())) }",
        ] {
            writer.ln(line)?;
        }
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(
            "pub extern \"C\" fn bindings_value_u128(low: u64, high: u64) -> *mut ValueHandle {",
        )?;
        writer.tab();
        writer.ln("into_value_handle(brec::csharp_feat::CSharpValue::U128(((high as u128) << 64) | low as u128))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln(
            "pub extern \"C\" fn bindings_value_i128(low: u64, high: i64) -> *mut ValueHandle {",
        )?;
        writer.tab();
        writer.ln("let raw = ((high as i128) << 64) | low as i128;")?;
        writer.ln("into_value_handle(brec::csharp_feat::CSharpValue::I128(raw))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_string(ptr: *const u8, len: usize) -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let bytes = match bytes_from_raw(ptr, len, \"value string\") { Ok(bytes) => bytes, Err(ptr) => return ptr };")?;
        writer.ln("match std::str::from_utf8(bytes) {")?;
        writer.tab();
        writer.ln("Ok(value) => into_value_handle(brec::csharp_feat::CSharpValue::String(value.to_owned())),")?;
        writer.ln("Err(err) => fail_ptr(format!(\"value string: invalid utf-8: {err}\")),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_bytes(ptr: *const u8, len: usize) -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let bytes = match bytes_from_raw(ptr, len, \"value bytes\") { Ok(bytes) => bytes, Err(ptr) => return ptr };")?;
        writer.ln("into_value_handle(brec::csharp_feat::CSharpValue::Bytes(bytes.to_vec()))")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_get_bool(handle: *const ValueHandle, out: *mut bool) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("if out.is_null() { set_last_error(\"value bool: output pointer is null\"); return false; }")?;
        writer.ln("match value_ref(handle, \"value bool\") { Ok(brec::csharp_feat::CSharpValue::Bool(value)) => { unsafe { *out = *value; } true }, Ok(other) => { set_last_error(format!(\"value bool: expected bool, got {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        for (name, rust_ty, variant) in [
            ("u8", "u8", "U8"),
            ("u16", "u16", "U16"),
            ("u32", "u32", "U32"),
            ("u64", "u64", "U64"),
            ("i8", "i8", "I8"),
            ("i16", "i16", "I16"),
            ("i32", "i32", "I32"),
            ("i64", "i64", "I64"),
            ("f32_bits", "u32", "F32Bits"),
            ("f64_bits", "u64", "F64Bits"),
        ] {
            writer.ln("")?;
            writer.ln("#[unsafe(no_mangle)]")?;
            writer.ln(format!("pub unsafe extern \"C\" fn bindings_value_get_{name}(handle: *const ValueHandle, out: *mut {rust_ty}) -> bool {{"))?;
            writer.tab();
            writer.ln("clear_last_error();")?;
            writer.ln(format!("if out.is_null() {{ set_last_error(\"value {name}: output pointer is null\"); return false; }}"))?;
            writer.ln(format!("match value_ref(handle, \"value {name}\") {{ Ok(brec::csharp_feat::CSharpValue::{variant}(value)) => {{ unsafe {{ *out = *value; }} true }}, Ok(other) => {{ set_last_error(format!(\"value {name}: unexpected kind: {{other:?}}\")); false }}, Err(()) => false }}"))?;
            writer.back();
            writer.ln("}")?;
        }
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_get_u128(handle: *const ValueHandle, low: *mut u64, high: *mut u64) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("if low.is_null() || high.is_null() { set_last_error(\"value u128: output pointer is null\"); return false; }")?;
        writer.ln("match value_ref(handle, \"value u128\") { Ok(brec::csharp_feat::CSharpValue::U128(value)) => { unsafe { *low = *value as u64; *high = (*value >> 64) as u64; } true }, Ok(other) => { set_last_error(format!(\"value u128: unexpected kind: {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_get_i128(handle: *const ValueHandle, low: *mut u64, high: *mut i64) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("if low.is_null() || high.is_null() { set_last_error(\"value i128: output pointer is null\"); return false; }")?;
        writer.ln("match value_ref(handle, \"value i128\") { Ok(brec::csharp_feat::CSharpValue::I128(value)) => { unsafe { *low = *value as u64; *high = (*value >> 64) as i64; } true }, Ok(other) => { set_last_error(format!(\"value i128: unexpected kind: {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_get_bytes(handle: *const ValueHandle, out_len: *mut usize) -> *mut u8 {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("match value_ref(handle, \"value bytes\") {")?;
        writer.tab();
        writer.ln("Ok(brec::csharp_feat::CSharpValue::Bytes(bytes)) => bytes_into_raw(bytes.clone(), out_len),")?;
        writer.ln("Ok(brec::csharp_feat::CSharpValue::String(value)) => bytes_into_raw(value.as_bytes().to_vec(), out_len),")?;
        writer
            .ln("Ok(other) => fail_ptr(format!(\"value bytes: unexpected kind: {other:?}\")),")?;
        writer.ln("Err(()) => std::ptr::null_mut(),")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_array_len(handle: *const ValueHandle) -> usize {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("match value_ref(handle, \"array len\") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => items.len(), Ok(other) => { set_last_error(format!(\"array len: unexpected kind: {other:?}\")); 0 }, Err(()) => 0 }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_array_get(handle: *const ValueHandle, index: usize) -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("match value_ref(handle, \"array get\") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => match items.get(index) { Some(value) => into_value_handle(value.clone()), None => fail_ptr(format!(\"array get: index {index} out of bounds\")) }, Ok(other) => fail_ptr(format!(\"array get: unexpected kind: {other:?}\")), Err(()) => std::ptr::null_mut() }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_array_push(handle: *mut ValueHandle, value: *const ValueHandle) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer
            .ln("let Ok(value) = value_ref(value, \"array push value\") else { return false; };")?;
        writer.ln("match value_mut(handle, \"array push\") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => { items.push(value.clone()); true }, Ok(other) => { set_last_error(format!(\"array push: unexpected kind: {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_object_has(handle: *const ValueHandle, key_ptr: *const u8, key_len: usize) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let key_bytes = match bytes_from_raw(key_ptr, key_len, \"object has key\") { Ok(bytes) => bytes, Err(_) => return false };")?;
        writer.ln("let Ok(key) = std::str::from_utf8(key_bytes) else { set_last_error(\"object has key: invalid utf-8\"); return false; };")?;
        writer.ln("match value_ref(handle, \"object has\") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => obj.contains_key(key), Ok(other) => { set_last_error(format!(\"object has: unexpected kind: {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_object_get(handle: *const ValueHandle, key_ptr: *const u8, key_len: usize) -> *mut ValueHandle {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let key_bytes = match bytes_from_raw(key_ptr, key_len, \"object get key\") { Ok(bytes) => bytes, Err(ptr) => return ptr };")?;
        writer.ln("let key = match std::str::from_utf8(key_bytes) { Ok(key) => key, Err(err) => return fail_ptr(format!(\"object get key: invalid utf-8: {err}\")) };")?;
        writer.ln("match value_ref(handle, \"object get\") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => match obj.get(key) { Some(value) => into_value_handle(value.clone()), None => fail_ptr(format!(\"object get: missing key {key}\")) }, Ok(other) => fail_ptr(format!(\"object get: unexpected kind: {other:?}\")), Err(()) => std::ptr::null_mut() }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[unsafe(no_mangle)]")?;
        writer.ln("pub unsafe extern \"C\" fn bindings_value_object_put(handle: *mut ValueHandle, key_ptr: *const u8, key_len: usize, value: *const ValueHandle) -> bool {")?;
        writer.tab();
        writer.ln("clear_last_error();")?;
        writer.ln("let key_bytes = match bytes_from_raw(key_ptr, key_len, \"object put key\") { Ok(bytes) => bytes, Err(_) => return false };")?;
        writer.ln("let Ok(key) = std::str::from_utf8(key_bytes) else { set_last_error(\"object put key: invalid utf-8\"); return false; };")?;
        writer
            .ln("let Ok(value) = value_ref(value, \"object put value\") else { return false; };")?;
        writer.ln("match value_mut(handle, \"object put\") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => { obj.insert(key.to_owned(), value.clone()); true }, Ok(other) => { set_last_error(format!(\"object put: unexpected kind: {other:?}\")); false }, Err(()) => false }")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
