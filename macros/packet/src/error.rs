use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("As Block can be used only structs")]
    StructNotFound,
    #[error("Only named fields of struct are supported")]
    NamedFieldsNotFound,
    #[error("Cannot extract identificator")]
    FailExtractIdent,
    #[error("Generic types are not supported")]
    GenericTypesNotSupported,
    #[error("Unsupported type")]
    UnsupportedType,
    #[error("Referred types are unsupported")]
    ReferenceUnsupported,
    #[error("Unsupported field type: {0}")]
    UnsupportedFieldType(String),
    #[error("Missed array size")]
    MissedArraySize,
    #[error("Fail parse generic argument")]
    FailParseGenericArg,
    #[error("Only single generic argument is supported")]
    OnlySingleGenericArg,
    #[error("{0} is reserved field name")]
    ReservedFieldName(String),
    #[error("Only primite types are supported in the context of slice")]
    UnsupportedTypeInSlice,

    #[error("Cannot detect attribute")]
    NoSuitableAttr,
    #[error("Attribute isn't supported")]
    UnsupportedAttr,
    #[error("Cannot parse attribute; unexpected attribute type")]
    UnexpectedAttrType,
    #[error("Missed name of enum type")]
    LinkingRequiresEnumName,
}
