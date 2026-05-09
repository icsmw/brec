use super::names::TypeNames;
use crate::*;
use brec_scheme::{PayloadTy, SchemeFieldType, SchemePayloadField};

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

    pub(super) fn field_type(&self, owner: &str, ty: &SchemeFieldType) -> Result<Type, Error> {
        match ty {
            SchemeFieldType::Payload(ty) => Ok(self.payload_type(ty)),
            other => Err(Error::InvalidScheme(format!(
                "payload field in {} contains non-payload type: {:?}",
                owner, other
            ))),
        }
    }

    pub(super) fn payload_type(&self, ty: &PayloadTy) -> Type {
        Type::from(ty).resolve_named(&|name| self.names.resolve(name).to_owned())
    }
}
