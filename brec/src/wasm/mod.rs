mod error;
mod packet;

use crate::Error;
pub use error::*;
pub use packet::*;
use wasm_bindgen::{JsCast, JsValue};

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Clone, Copy, Debug)]
pub enum WasmFieldHint {
    Bool,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    I64,
    U64,
    I128,
    U128,
    String,
    F64,
    Vec,
    Option,
    Blob,
    Blocks,
    Payload,
    Object,
}

#[inline]
pub fn from_value<T: JsCast>(hint: WasmFieldHint, value: JsValue) -> Result<T, Error> {
    value
        .dyn_into::<T>()
        .map_err(|_| WasmError::invalid_field(hint, "type conversion failed"))
}

#[inline]
pub fn from_value_name<T: JsCast>(name: impl Into<String>, value: JsValue) -> Result<T, Error> {
    value
        .dyn_into::<T>()
        .map_err(|_| WasmError::invalid_field_name(name, "type conversion failed"))
}

#[cfg(all(test, target_arch = "wasm32", feature = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn from_value_and_from_value_name_success_for_object() {
        let value: JsValue = js_sys::Object::new().into();

        let obj_a: js_sys::Object =
            from_value(WasmFieldHint::Object, value.clone()).expect("from_value object");
        let obj_b: js_sys::Object = from_value_name("obj", value).expect("from_value_name object");

        assert!(obj_a.is_object());
        assert!(obj_b.is_object());
    }

    #[wasm_bindgen_test]
    fn from_value_reports_type_conversion_error() {
        let value: JsValue = js_sys::Object::new().into();
        let err = from_value::<js_sys::Array>(WasmFieldHint::Vec, value).expect_err("must fail");

        match err {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "vec");
                assert!(message.contains("type conversion failed"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[wasm_bindgen_test]
    fn from_value_name_reports_type_conversion_error() {
        let value = JsValue::from_f64(123.0);
        let err = from_value_name::<js_sys::Object>("payload", value).expect_err("must fail");

        match err {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "payload");
                assert!(message.contains("type conversion failed"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[wasm_bindgen_test]
    fn wasm_error_helpers_wrap_field_and_name() {
        let by_hint = WasmError::invalid_field(WasmFieldHint::U64, "bad");
        match by_hint {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "u64");
                assert_eq!(message, "bad");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }

        let by_name = WasmError::invalid_field_name("my_field", "oops");
        match by_name {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "my_field");
                assert_eq!(message, "oops");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
