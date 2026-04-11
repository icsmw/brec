use super::WasmFieldHint;
use thiserror::Error;

/// Error details for Rust <-> JavaScript conversion in `wasm` helpers.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WasmError {
    /// The provided JavaScript value is not a valid object for conversion.
    #[error("Invalid JS object: {0}")]
    InvalidObject(String),
    /// The JavaScript object does not have an expected field.
    #[error("Missing field: {0}")]
    MissingField(String),
    /// A field value could not be converted to the target Rust type.
    #[error("Invalid field value for {0}: {1}")]
    InvalidField(String, String),
    /// The provided JavaScript object shape is invalid for an aggregator.
    #[error("Invalid aggregator object shape: {0}")]
    InvalidAggregatorShape(String),
}

impl WasmError {
    #[inline]
    pub fn invalid_field(hint: WasmFieldHint, err: impl ToString) -> crate::Error {
        Self::InvalidField(hint.id().to_string(), err.to_string()).into()
    }

    #[inline]
    pub fn invalid_field_name(name: impl Into<String>, err: impl ToString) -> crate::Error {
        Self::InvalidField(name.into(), err.to_string()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn wasm_error_display_messages_are_stable() {
        let invalid_object = WasmError::InvalidObject("bad root".to_owned());
        assert_eq!(invalid_object.to_string(), "Invalid JS object: bad root");

        let missing = WasmError::MissingField("payload".to_owned());
        assert_eq!(missing.to_string(), "Missing field: payload");

        let invalid_field = WasmError::InvalidField("u64".to_owned(), "expected BigInt".to_owned());
        assert_eq!(
            invalid_field.to_string(),
            "Invalid field value for u64: expected BigInt"
        );

        let shape = WasmError::InvalidAggregatorShape("missing packets".to_owned());
        assert_eq!(
            shape.to_string(),
            "Invalid aggregator object shape: missing packets"
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn invalid_field_helpers_wrap_into_crate_error() {
        let by_hint = WasmError::invalid_field(WasmFieldHint::U16, "not a number");
        match by_hint {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "u16");
                assert_eq!(message, "not a number");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }

        let by_name = WasmError::invalid_field_name("custom_field", "broken");
        match by_name {
            Error::Wasm(WasmError::InvalidField(name, message)) => {
                assert_eq!(name, "custom_field");
                assert_eq!(message, "broken");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
