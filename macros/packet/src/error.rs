use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("As Packet can be used only structs")]
    StructNotFound,
    #[error("Only named fields of struct are supported")]
    NamedFieldsNotFound,
    #[error("Cannot extract identificator")]
    FailExtractIdent,
    #[error("Unsupported type")]
    UnsupportedType,
    #[error("Unsupported field type: {0}")]
    UnsupportedFieldType(String),
    #[error("Missed array size")]
    MissedArraySize,
    #[error("Fail parse generic argument")]
    FailParseGenericArg,
    #[error("Only single generic argument is supported")]
    OnlySingleGenericArg,
    #[error("Generic type isn't supported for this type")]
    GenericNotSupported,

    #[error("Missed name of enum type")]
    LinkingRequiresEnumName,
}
