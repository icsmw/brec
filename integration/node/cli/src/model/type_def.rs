use super::resolver::Resolver;
use crate::*;
use brec_scheme::{SchemePayload, SchemePayloadField, SchemePayloadVariant, SchemeType};

pub struct TypeDef {
    name: String,
    body: TypeBody,
}

enum TypeBody {
    Empty,
    Struct(StructFields),
    Enum(Vec<EnumVariant>),
}

enum StructFields {
    Named(Vec<Field>),
    Tuple(Vec<Type>),
}

struct EnumVariant(Type);

enum VariantBody {
    Unit,
    Named(Vec<Field>),
    Single(Type),
    Tuple(Vec<Type>),
}

impl TypeDef {
    pub(super) fn from_payload(
        payload: &SchemePayload,
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        Self::from_parts(
            &payload.fullname,
            &payload.fields,
            &payload.variants,
            resolver,
        )
    }

    pub(super) fn from_scheme_type(
        scheme_type: &SchemeType,
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        Self::from_parts(
            &scheme_type.fullname,
            &scheme_type.fields,
            &scheme_type.variants,
            resolver,
        )
    }

    pub fn declaration(&self) -> Declaration {
        match &self.body {
            TypeBody::Empty => {
                Declaration::Type(TypeAlias::new(self.name.clone(), Type::empty_object()))
            }
            TypeBody::Struct(StructFields::Named(fields)) => {
                Declaration::Interface(Interface::new(self.name.clone(), fields.clone()))
            }
            TypeBody::Struct(StructFields::Tuple(items)) => Declaration::Type(TypeAlias::new(
                self.name.clone(),
                Type::Tuple(items.clone()),
            )),
            TypeBody::Enum(variants) => Declaration::Type(TypeAlias::new(
                self.name.clone(),
                Type::Union(variants.iter().map(EnumVariant::ty).collect()),
            )),
        }
    }

    fn from_parts(
        name: &str,
        fields: &[SchemePayloadField],
        variants: &[SchemePayloadVariant],
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        let body = match (fields.is_empty(), variants.is_empty()) {
            (true, true) => TypeBody::Empty,
            (false, true) => TypeBody::Struct(StructFields::from_scheme(name, fields, resolver)?),
            (true, false) => TypeBody::Enum(
                variants
                    .iter()
                    .map(|variant| EnumVariant::from_scheme(name, variant, resolver))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            (false, false) => {
                return Err(Error::InvalidScheme(format!(
                    "type {} contains both fields and variants",
                    name
                )));
            }
        };
        Ok(Self {
            name: name.to_owned(),
            body,
        })
    }
}

impl SourceWritable for TypeDef {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.declaration().write(writer)
    }
}

impl StructFields {
    fn from_scheme(
        owner: &str,
        fields: &[SchemePayloadField],
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        if fields.iter().all(|field| field.name.is_some()) {
            return Ok(Self::Named(
                fields
                    .iter()
                    .map(|field| resolver.named_field(owner, field))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        if fields.iter().all(|field| field.name.is_none()) {
            return Ok(Self::Tuple(
                fields
                    .iter()
                    .map(|field| resolver.field_type(owner, &field.ty))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        Err(Error::InvalidScheme(format!(
            "type {} mixes named and unnamed fields",
            owner
        )))
    }
}

impl EnumVariant {
    fn from_scheme(
        owner: &str,
        variant: &SchemePayloadVariant,
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        let label = format!("{owner}::{}", variant.name);
        let body = VariantBody::from_scheme(&label, &variant.fields, resolver)?;
        Ok(Self(Type::object(vec![Field::required(
            &variant.name,
            body.ty(),
        )?])))
    }

    fn ty(&self) -> Type {
        self.0.clone()
    }
}

impl VariantBody {
    fn ty(&self) -> Type {
        match self {
            Self::Unit => Type::Null,
            Self::Named(fields) => Type::object(fields.clone()),
            Self::Single(ty) => ty.clone(),
            Self::Tuple(items) => Type::Tuple(items.clone()),
        }
    }

    fn from_scheme(
        owner: &str,
        fields: &[SchemePayloadField],
        resolver: &Resolver<'_>,
    ) -> Result<Self, Error> {
        if fields.is_empty() {
            return Ok(Self::Unit);
        }

        if fields.iter().all(|field| field.name.is_some()) {
            return Ok(Self::Named(
                fields
                    .iter()
                    .map(|field| resolver.named_field(owner, field))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        if fields.iter().all(|field| field.name.is_none()) {
            if fields.len() == 1 {
                return Ok(Self::Single(resolver.field_type(owner, &fields[0].ty)?));
            }
            return Ok(Self::Tuple(
                fields
                    .iter()
                    .map(|field| resolver.field_type(owner, &field.ty))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        Err(Error::InvalidScheme(format!(
            "payload enum variant {} mixes named and unnamed fields",
            owner
        )))
    }
}
