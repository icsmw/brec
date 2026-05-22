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
        writer.block(
            r#"
use protocol::{Block, Packet, Payload};
use std::cell::RefCell;
use std::ffi::{CString, c_char};
use std::collections::BTreeMap;

pub struct ValueHandle {
	value: brec::csharp_feat::CSharpValue,
}

thread_local! {
	static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn sanitize_nul_bytes(input: impl Into<String>) -> String {
	input.into().replace('\0', " ")
}

fn set_last_error(message: impl Into<String>) {
	let text = sanitize_nul_bytes(message);
	LAST_ERROR.with(|cell| {
		*cell.borrow_mut() = CString::new(text).ok();
	});
}

fn clear_last_error() {
	LAST_ERROR.with(|cell| {
		*cell.borrow_mut() = None;
	});
}

fn fail_ptr<T>(message: impl Into<String>) -> *mut T {
	set_last_error(message);
	std::ptr::null_mut()
}

fn into_value_handle(value: brec::csharp_feat::CSharpValue) -> *mut ValueHandle {
	Box::into_raw(Box::new(ValueHandle { value }))
}

fn bytes_into_raw(mut bytes: Vec<u8>, out_len: *mut usize) -> *mut u8 {
	if out_len.is_null() {
		return fail_ptr("output length pointer is null");
	}
	let ptr = bytes.as_mut_ptr();
	let len = bytes.len();
	unsafe { *out_len = len; }
	std::mem::forget(bytes);
	ptr
}

fn bytes_from_raw<'a>(
"#,
        )?;
        writer.block(
            r#"
	bytes_ptr: *const u8,
	bytes_len: usize,
	op: &str,
) -> Result<&'a [u8], *mut ValueHandle> {
	if bytes_ptr.is_null() && bytes_len != 0 {
		return Err(fail_ptr(format!("{op}: input pointer is null")));
	}
	if bytes_len == 0 {
		return Ok(&[]);
	}
	Ok(unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) })
}

fn value_handle_ref<'a>(
	handle: *const ValueHandle,
	_out_len: *mut usize,
	op: &str,
) -> Result<&'a ValueHandle, *mut u8> {
	if handle.is_null() {
		set_last_error(format!("{op}: handle is null"));
		return Err(std::ptr::null_mut());
	}
	Ok(unsafe { &*handle })
}

#[unsafe(no_mangle)]
pub extern "C" fn bindings_last_error_message() -> *const c_char {
	LAST_ERROR.with(|cell| {
		if let Some(msg) = cell.borrow().as_ref() {
			msg.as_ptr()
		} else {
			std::ptr::null()
		}
	})
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_free(handle: *mut ValueHandle) {
	if handle.is_null() {
		return;
	}
	unsafe { let _ = Box::from_raw(handle); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_bytes_free(ptr: *mut u8, len: usize) {
	if ptr.is_null() {
		return;
	}
	unsafe {
		let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, len);
		let _ = Box::<[u8]>::from_raw(slice_ptr);
	}
}

fn value_ref<'a>(handle: *const ValueHandle, op: &str) -> Result<&'a brec::csharp_feat::CSharpValue, ()> {
	if handle.is_null() {
		set_last_error(format!("{op}: handle is null"));
		return Err(());
	}
	Ok(unsafe { &(*handle).value })
}

fn value_mut<'a>(handle: *mut ValueHandle, op: &str) -> Result<&'a mut brec::csharp_feat::CSharpValue, ()> {
	if handle.is_null() {
		set_last_error(format!("{op}: handle is null"));
		return Err(());
	}
	Ok(unsafe { &mut (*handle).value })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_kind(handle: *const ValueHandle) -> i32 {
	clear_last_error();
	let Ok(value) = value_ref(handle, "value kind") else { return -1; };
	match value {
		brec::csharp_feat::CSharpValue::Null => 0,
		brec::csharp_feat::CSharpValue::Bool(_) => 1,
		brec::csharp_feat::CSharpValue::U8(_) => 2,
		brec::csharp_feat::CSharpValue::U16(_) => 3,
		brec::csharp_feat::CSharpValue::U32(_) => 4,
		brec::csharp_feat::CSharpValue::U64(_) => 5,
		brec::csharp_feat::CSharpValue::U128(_) => 6,
		brec::csharp_feat::CSharpValue::I8(_) => 7,
		brec::csharp_feat::CSharpValue::I16(_) => 8,
		brec::csharp_feat::CSharpValue::I32(_) => 9,
		brec::csharp_feat::CSharpValue::I64(_) => 10,
		brec::csharp_feat::CSharpValue::I128(_) => 11,
		brec::csharp_feat::CSharpValue::F32Bits(_) => 12,
		brec::csharp_feat::CSharpValue::F64Bits(_) => 13,
		brec::csharp_feat::CSharpValue::String(_) => 14,
		brec::csharp_feat::CSharpValue::Bytes(_) => 15,
		brec::csharp_feat::CSharpValue::Array(_) => 16,
		brec::csharp_feat::CSharpValue::Object(_) => 17,
	}
}
"#,
        )?;
        writer.block(
            r#"
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_null() -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Null) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_bool(value: bool) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Bool(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_u8(value: u8) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U8(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_u16(value: u16) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U16(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_u32(value: u32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U32(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_u64(value: u64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::U64(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_i8(value: i8) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I8(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_i16(value: i16) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I16(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_i32(value: i32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I32(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_i64(value: i64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::I64(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_f32_bits(value: u32) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::F32Bits(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_f64_bits(value: u64) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::F64Bits(value)) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_array(capacity: usize) -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Array(Vec::with_capacity(capacity))) }
#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_object() -> *mut ValueHandle { into_value_handle(brec::csharp_feat::CSharpValue::Object(BTreeMap::new())) }

#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_u128(low: u64, high: u64) -> *mut ValueHandle {
	into_value_handle(brec::csharp_feat::CSharpValue::U128(((high as u128) << 64) | low as u128))
}

#[unsafe(no_mangle)]
pub extern "C" fn bindings_value_i128(low: u64, high: i64) -> *mut ValueHandle {
	let raw = ((high as i128) << 64) | low as i128;
	into_value_handle(brec::csharp_feat::CSharpValue::I128(raw))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_string(ptr: *const u8, len: usize) -> *mut ValueHandle {
	clear_last_error();
	let bytes = match bytes_from_raw(ptr, len, "value string") { Ok(bytes) => bytes, Err(ptr) => return ptr };
	match std::str::from_utf8(bytes) {
		Ok(value) => into_value_handle(brec::csharp_feat::CSharpValue::String(value.to_owned())),
		Err(err) => fail_ptr(format!("value string: invalid utf-8: {err}")),
	}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_bytes(ptr: *const u8, len: usize) -> *mut ValueHandle {
	clear_last_error();
	let bytes = match bytes_from_raw(ptr, len, "value bytes") { Ok(bytes) => bytes, Err(ptr) => return ptr };
	into_value_handle(brec::csharp_feat::CSharpValue::Bytes(bytes.to_vec()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_get_bool(handle: *const ValueHandle, out: *mut bool) -> bool {
	clear_last_error();
	if out.is_null() { set_last_error("value bool: output pointer is null"); return false; }
	match value_ref(handle, "value bool") { Ok(brec::csharp_feat::CSharpValue::Bool(value)) => { unsafe { *out = *value; } true }, Ok(other) => { set_last_error(format!("value bool: expected bool, got {other:?}")); false }, Err(()) => false }
}
"#,
        )?;
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
            writer.block(format!(
                r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_get_{name}(handle: *const ValueHandle, out: *mut {rust_ty}) -> bool {{
	clear_last_error();
	if out.is_null() {{ set_last_error("value {name}: output pointer is null"); return false; }}
	match value_ref(handle, "value {name}") {{ Ok(brec::csharp_feat::CSharpValue::{variant}(value)) => {{ unsafe {{ *out = *value; }} true }}, Ok(other) => {{ set_last_error(format!("value {name}: unexpected kind: {{other:?}}")); false }}, Err(()) => false }}
}}
"#
            ))?;
        }
        writer.block(
            r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_get_u128(handle: *const ValueHandle, low: *mut u64, high: *mut u64) -> bool {
	clear_last_error();
	if low.is_null() || high.is_null() { set_last_error("value u128: output pointer is null"); return false; }
	match value_ref(handle, "value u128") { Ok(brec::csharp_feat::CSharpValue::U128(value)) => { unsafe { *low = *value as u64; *high = (*value >> 64) as u64; } true }, Ok(other) => { set_last_error(format!("value u128: unexpected kind: {other:?}")); false }, Err(()) => false }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_get_i128(handle: *const ValueHandle, low: *mut u64, high: *mut i64) -> bool {
	clear_last_error();
	if low.is_null() || high.is_null() { set_last_error("value i128: output pointer is null"); return false; }
	match value_ref(handle, "value i128") { Ok(brec::csharp_feat::CSharpValue::I128(value)) => { unsafe { *low = *value as u64; *high = (*value >> 64) as i64; } true }, Ok(other) => { set_last_error(format!("value i128: unexpected kind: {other:?}")); false }, Err(()) => false }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_get_bytes(handle: *const ValueHandle, out_len: *mut usize) -> *mut u8 {
	clear_last_error();
	match value_ref(handle, "value bytes") {
		Ok(brec::csharp_feat::CSharpValue::Bytes(bytes)) => bytes_into_raw(bytes.clone(), out_len),
		Ok(brec::csharp_feat::CSharpValue::String(value)) => bytes_into_raw(value.as_bytes().to_vec(), out_len),
		Ok(other) => fail_ptr(format!("value bytes: unexpected kind: {other:?}")),
		Err(()) => std::ptr::null_mut(),
	}
}
"#,
        )?;
        writer.block(
            r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_array_len(handle: *const ValueHandle) -> usize {
	clear_last_error();
	match value_ref(handle, "array len") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => items.len(), Ok(other) => { set_last_error(format!("array len: unexpected kind: {other:?}")); 0 }, Err(()) => 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_array_get(handle: *const ValueHandle, index: usize) -> *mut ValueHandle {
	clear_last_error();
	match value_ref(handle, "array get") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => match items.get(index) { Some(value) => into_value_handle(value.clone()), None => fail_ptr(format!("array get: index {index} out of bounds")) }, Ok(other) => fail_ptr(format!("array get: unexpected kind: {other:?}")), Err(()) => std::ptr::null_mut() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_array_push(handle: *mut ValueHandle, value: *const ValueHandle) -> bool {
	clear_last_error();
	let Ok(value) = value_ref(value, "array push value") else { return false; };
	match value_mut(handle, "array push") { Ok(brec::csharp_feat::CSharpValue::Array(items)) => { items.push(value.clone()); true }, Ok(other) => { set_last_error(format!("array push: unexpected kind: {other:?}")); false }, Err(()) => false }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_object_has(handle: *const ValueHandle, key_ptr: *const u8, key_len: usize) -> bool {
	clear_last_error();
	let key_bytes = match bytes_from_raw(key_ptr, key_len, "object has key") { Ok(bytes) => bytes, Err(_) => return false };
	let Ok(key) = std::str::from_utf8(key_bytes) else { set_last_error("object has key: invalid utf-8"); return false; };
	match value_ref(handle, "object has") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => obj.contains_key(key), Ok(other) => { set_last_error(format!("object has: unexpected kind: {other:?}")); false }, Err(()) => false }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_object_get(handle: *const ValueHandle, key_ptr: *const u8, key_len: usize) -> *mut ValueHandle {
	clear_last_error();
	let key_bytes = match bytes_from_raw(key_ptr, key_len, "object get key") { Ok(bytes) => bytes, Err(ptr) => return ptr };
	let key = match std::str::from_utf8(key_bytes) { Ok(key) => key, Err(err) => return fail_ptr(format!("object get key: invalid utf-8: {err}")) };
	match value_ref(handle, "object get") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => match obj.get(key) { Some(value) => into_value_handle(value.clone()), None => fail_ptr(format!("object get: missing key {key}")) }, Ok(other) => fail_ptr(format!("object get: unexpected kind: {other:?}")), Err(()) => std::ptr::null_mut() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_value_object_put(handle: *mut ValueHandle, key_ptr: *const u8, key_len: usize, value: *const ValueHandle) -> bool {
	clear_last_error();
	let key_bytes = match bytes_from_raw(key_ptr, key_len, "object put key") { Ok(bytes) => bytes, Err(_) => return false };
	let Ok(key) = std::str::from_utf8(key_bytes) else { set_last_error("object put key: invalid utf-8"); return false; };
	let Ok(value) = value_ref(value, "object put value") else { return false; };
	match value_mut(handle, "object put") { Ok(brec::csharp_feat::CSharpValue::Object(obj)) => { obj.insert(key.to_owned(), value.clone()); true }, Ok(other) => { set_last_error(format!("object put: unexpected kind: {other:?}")); false }, Err(()) => false }
}
"#,
        )?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
