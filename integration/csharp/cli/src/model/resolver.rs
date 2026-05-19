use super::CSharpType;
use super::field::FieldDef;
use super::ident::csharp_property_name;
use super::names::TypeNames;
use crate::Error;
use brec_scheme::{SchemeFieldType, SchemePayloadField};

pub(super) fn named_field(
    owner: &str,
    field: &SchemePayloadField,
    names: &TypeNames,
) -> Result<FieldDef, Error> {
    let name = field.name.as_deref().ok_or_else(|| {
        Error::InvalidScheme(format!("type {} mixes named and unnamed fields", owner))
    })?;
    let (ty, nullable) = payload_field_type(owner, &field.ty, names)?;
    Ok(FieldDef {
        key: name.to_owned(),
        name: csharp_property_name(name),
        ty,
        nullable,
    })
}

pub(super) fn payload_field_type(
    owner: &str,
    ty: &SchemeFieldType,
    names: &TypeNames,
) -> Result<(CSharpType, bool), Error> {
    match ty {
        SchemeFieldType::Payload(ty) => Ok(CSharpType::from_payload_ty(ty, names)),
        other => Err(Error::InvalidScheme(format!(
            "payload field in {} contains non-payload type: {:?}",
            owner, other
        ))),
    }
}
