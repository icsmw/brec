use protocol::Packet;
use std::cell::RefCell;
use std::ffi::{CString, c_char};

pub struct PacketHandle {
    value: brec::CSharpValue,
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
pub unsafe extern "C" fn bindings_packet_decode(
    bytes_ptr: *const u8,
    bytes_len: usize,
) -> *mut PacketHandle {
    clear_last_error();

    if bytes_ptr.is_null() && bytes_len != 0 {
        set_last_error("decode packet: input pointer is null");
        return std::ptr::null_mut();
    }

    let bytes = if bytes_len == 0 {
        &[][..]
    } else {
        // SAFETY: validated null/len pair above, caller guarantees valid memory region.
        unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) }
    };

    let mut ctx = ();
    match Packet::decode_csharp(bytes, &mut ctx) {
        Ok(value) => Box::into_raw(Box::new(PacketHandle { value })),
        Err(err) => {
            set_last_error(format!("decode packet failed: {err}"));
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_packet_encode(
    handle: *const PacketHandle,
    out_len: *mut usize,
) -> *mut u8 {
    clear_last_error();

    if handle.is_null() {
        set_last_error("encode packet: handle is null");
        return std::ptr::null_mut();
    }
    if out_len.is_null() {
        set_last_error("encode packet: out_len pointer is null");
        return std::ptr::null_mut();
    }

    // SAFETY: pointers are checked above.
    let handle_ref = unsafe { &*handle };

    let mut out = Vec::new();
    let mut ctx = ();
    if let Err(err) = Packet::encode_csharp(handle_ref.value.clone(), &mut out, &mut ctx) {
        set_last_error(format!("encode packet failed: {err}"));
        return std::ptr::null_mut();
    }

    let mut boxed = out.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    // SAFETY: out_len is checked non-null above.
    unsafe { *out_len = len };
    std::mem::forget(boxed);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_packet_free(handle: *mut PacketHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: pointer originates from Box::into_raw in bindings_packet_decode.
    unsafe {
        let _ = Box::from_raw(handle);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_bytes_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: pointer/len must come from bindings_packet_encode allocation.
    unsafe {
        let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, len);
        let _ = Box::<[u8]>::from_raw(slice_ptr);
    }
}
