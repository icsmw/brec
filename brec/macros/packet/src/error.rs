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
    #[error("Referred types are unsupported")]
    ReferenceUnsupported,
    #[error("Missed array size")]
    MissedArraySize,
    #[error("{0} is reserved field name")]
    ReservedFieldName(String),

    #[error("Cannot detect attribute")]
    NoSuitableAttr,
    #[error("Attribute isn't supported")]
    UnsupportedAttr,
    #[error("Missed name of enum type")]
    LinkingRequiresEnumName,

    #[error("Fail to access to collector")]
    NoAccessToCollector,

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Var Error: {0}")]
    Var(#[from] std::env::VarError),
}
