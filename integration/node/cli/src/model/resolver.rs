use super::names::TypeNames;
use crate::*;
use brec_scheme::{PayloadTy, SchemeFieldType, SchemePayloadField};

/// Converts scheme field types into TypeScript model types.
///
/// `Resolver` owns the boundary between raw scheme names and exported
/// TypeScript identifiers. Keeping this lookup here prevents individual type
/// writers from needing to know how Rust paths are normalized.
pub(super) struct Resolver<'a> {
    names: &'a TypeNames,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(names: &'a TypeNames) -> Self {
        Self { names }
    }

    pub(super) fn named_field(
        &self,
        owner: &str,
        field: &SchemePayloadField,
    ) -> Result<Field, Error> {
        let name = field.name.as_deref().ok_or_else(|| {
            Error::InvalidScheme(format!("type {} mixes named and unnamed fields", owner))
        })?;

        match &field.ty {
            SchemeFieldType::Payload(PayloadTy::Option(inner)) => {
                Field::optional(name, self.payload_type(inner))
            }
            SchemeFieldType::Payload(ty) => Field::required(name, self.payload_type(ty)),
            other => Err(Error::InvalidScheme(format!(
                "payload field in {} contains non-payload type: {:?}",
                owner, other
            ))),
        }
    }

    /// Converts a raw payload field type into a TypeScript type.
    ///
    /// Scheme payload fields should only contain `SchemeFieldType::Payload`.
    /// Seeing a block field here means the generator input is inconsistent, so
    /// the conversion fails with the owning type in the message.
    pub(super) fn field_type(&self, owner: &str, ty: &SchemeFieldType) -> Result<Type, Error> {
        match ty {
            SchemeFieldType::Payload(ty) => Ok(self.payload_type(ty)),
            other => Err(Error::InvalidScheme(format!(
                "payload field in {} contains non-payload type: {:?}",
                owner, other
            ))),
        }
    }

    /// Resolves `Struct` and `Enum` references to the exported TypeScript names.
    pub(super) fn payload_type(&self, ty: &PayloadTy) -> Type {
        Type::from(ty).resolve_named(&|name| self.names.resolve(name).to_owned())
    }
}
