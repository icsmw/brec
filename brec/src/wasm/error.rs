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
