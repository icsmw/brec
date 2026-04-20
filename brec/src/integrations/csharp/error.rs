use super::CSharpFieldHint;
use thiserror::Error;

/// Error details for Rust <-> C# conversion in `csharp` helpers.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CSharpError {
    /// The provided C# value is not a valid object for conversion.
    #[error("Invalid C# object: {0}")]
    InvalidObject(String),
    /// The C# object does not have an expected field.
    #[error("Missing field: {0}")]
    MissingField(String),
    /// A field value could not be converted to the target Rust type.
    #[error("Invalid field value for {0}: {1}")]
    InvalidField(String, String),
    /// The provided C# object shape is invalid for an aggregator.
    #[error("Invalid aggregator object shape: {0}")]
    InvalidAggregatorShape(String),
}

impl CSharpError {
    #[inline]
    pub fn invalid_field(hint: CSharpFieldHint, err: impl ToString) -> crate::Error {
        Self::InvalidField(hint.id().to_string(), err.to_string()).into()
    }

    #[inline]
    pub fn invalid_field_name(name: impl Into<String>, err: impl ToString) -> crate::Error {
        Self::InvalidField(name.into(), err.to_string()).into()
    }
}
