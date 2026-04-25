use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Cannot extract identificator")]
    FailExtractIdent,
    #[error("Cannot parse full path")]
    FailParseFullpath,
    #[error("Generic types are not supported")]
    GenericTypesNotSupported,
    #[error("Unsupported type")]
    UnsupportedType,
    #[error("Unsupported type to use with {0}")]
    NotSupportedBy(String),
    #[error("Referred types are unsupported")]
    ReferenceUnsupported,
    #[error("Missed array size")]
    MissedArraySize,
    #[error("{0} is reserved field name")]
    ReservedFieldName(String),
    #[error("{0} is unknown visibility")]
    FailParseVisibility(String),
    #[error("Fail parser derive: {0}")]
    FailParseDerive(String),

    #[error("Attribute isn't supported")]
    UnsupportedAttr,

    #[error("Fail to access to collector")]
    NoAccessToCollector,

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Var Error: {0}")]
    Var(#[from] std::env::VarError),
}
