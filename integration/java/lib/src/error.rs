use crate::JavaFieldHint;
use thiserror::Error;

/// Error details for Rust <-> Java conversion in `java` helpers.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum JavaError {
    /// The provided Java value is not a valid object for conversion.
    #[error("Invalid Java object: {0}")]
    InvalidObject(String),
    /// The Java object does not have an expected field.
    #[error("Missing field: {0}")]
    MissingField(String),
    /// A field value could not be converted to the target Rust type.
    #[error("Invalid field value for {0}: {1}")]
    InvalidField(String, String),
    /// The provided Java object shape is invalid for an aggregator.
    #[error("Invalid aggregator object shape: {0}")]
    InvalidAggregatorShape(String),
}

impl JavaError {
    #[inline]
    pub fn invalid_field(hint: JavaFieldHint, err: impl ToString) -> JavaError {
        Self::InvalidField(hint.id().to_string(), err.to_string())
    }

    #[inline]
    pub fn invalid_field_name(name: impl Into<String>, err: impl ToString) -> JavaError {
        Self::InvalidField(name.into(), err.to_string())
    }
}
