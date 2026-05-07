use crate::FormatterWritable;

use super::field::Field;
use brec_scheme::{BlockTy, PayloadTy};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Number,
    BigInt,
    String,
    Boolean,
    Null,
    Undefined,
    Unknown,
    Named(String),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Union(Vec<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Object(Vec<Field>),
}

impl Type {
    pub fn object(fields: Vec<Field>) -> Self {
        Self::Object(fields)
    }

    pub fn empty_object() -> Self {
        Self::Object(Vec::new())
    }

    pub fn array(inner: Type) -> Self {
        Self::Array(Box::new(inner))
    }

    pub fn union(items: impl IntoIterator<Item = Type>) -> Self {
        let mut flattened = Vec::new();
        for item in items {
            match item {
                Self::Union(inner) => flattened.extend(inner),
                other => flattened.push(other),
            }
        }

        let mut unique = Vec::new();
        for item in flattened {
            if !unique.contains(&item) {
                unique.push(item);
            }
        }

        let mut unique = unique.into_iter();
        match (unique.next(), unique.next()) {
            (None, _) => Self::Unknown,
            (Some(item), None) => item,
            (Some(first), Some(second)) => {
                let mut items = vec![first, second];
                items.extend(unique);
                Self::Union(items)
            }
        }
    }

    pub fn optional_value(inner: Type) -> Self {
        Self::union([inner, Self::Undefined])
    }

    pub fn resolve_named<F>(self, resolver: &F) -> Self
    where
        F: Fn(&str) -> String,
    {
        match self {
            Self::Named(name) => Self::Named(resolver(&name)),
            Self::Array(inner) => Self::Array(Box::new(inner.resolve_named(resolver))),
            Self::Tuple(items) => Self::Tuple(
                items
                    .into_iter()
                    .map(|item| item.resolve_named(resolver))
                    .collect(),
            ),
            Self::Union(items) => Self::Union(
                items
                    .into_iter()
                    .map(|item| item.resolve_named(resolver))
                    .collect(),
            ),
            Self::Map(key, value) => Self::Map(
                Box::new(key.resolve_named(resolver)),
                Box::new(value.resolve_named(resolver)),
            ),
            Self::Set(inner) => Self::Set(Box::new(inner.resolve_named(resolver))),
            other => other,
        }
    }
}

impl FormatterWritable for Type {
    fn write(&self, writer: &mut crate::FormatterWriter) -> fmt::Result {
        match self {
            Self::Number => writer.write("number"),
            Self::BigInt => writer.write("bigint"),
            Self::String => writer.write("string"),
            Self::Boolean => writer.write("boolean"),
            Self::Null => writer.write("null"),
            Self::Undefined => writer.write("undefined"),
            Self::Unknown => writer.write("unknown"),
            Self::Named(name) => writer.write(name),
            Self::Array(inner) => {
                match inner.as_ref() {
                    Self::Union(_) => {
                        writer.write("(")?;
                        inner.write(writer)?;
                        writer.write(")")?;
                    },
                    _ => {
                        inner.write(writer)?;
                    },
                }
                writer.write("[]")
            }
            Self::Tuple(items) => {
                writer.write("[")?;
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        writer.write(", ")?;
                    }
                    item.write(writer)?;
                }
                writer.write("]")
            }
            Self::Union(items) => {
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" | ")?;
                    }
                    item.write(writer)?;
                }
                Ok(())
            }
            Self::Map(key, value) => {
                writer.write("Map<")?;
                key.write(writer)?;
                writer.write(", ")?;
                value.write(writer)?;
                writer.write(">")
            }
            Self::Set(inner) => {
                writer.write("Set<")?;
                inner.write(writer)?;
                writer.write(">")
            },
            Self::Object(fields) => {
                if fields.is_empty() {
                    return writer.write("Record<string, never>");
                }
                writer.ln("{")?;
                writer.tab();
                for (_idx, field) in fields.iter().enumerate() {
                    field.write(writer)?;
                }
                writer.back();
                writer.ln("}")
            }
        }
    }
}

impl From<&BlockTy> for Type {
    fn from(value: &BlockTy) -> Self {
        match value {
            BlockTy::U8
            | BlockTy::U16
            | BlockTy::U32
            | BlockTy::I8
            | BlockTy::I16
            | BlockTy::I32
            | BlockTy::F32
            | BlockTy::LinkedToU8(_) => Self::Number,
            BlockTy::U64 | BlockTy::U128 | BlockTy::I64 | BlockTy::I128 | BlockTy::F64 => {
                Self::BigInt
            }
            BlockTy::Bool => Self::Boolean,
            BlockTy::Blob(_) => Self::array(Self::Number),
        }
    }
}

impl From<BlockTy> for Type {
    fn from(value: BlockTy) -> Self {
        Self::from(&value)
    }
}

impl From<&PayloadTy> for Type {
    fn from(value: &PayloadTy) -> Self {
        match value {
            PayloadTy::String => Self::String,
            PayloadTy::U8
            | PayloadTy::U16
            | PayloadTy::U32
            | PayloadTy::I8
            | PayloadTy::I16
            | PayloadTy::I32
            | PayloadTy::F32 => Self::Number,
            PayloadTy::U64
            | PayloadTy::U128
            | PayloadTy::I64
            | PayloadTy::I128
            | PayloadTy::F64 => Self::BigInt,
            PayloadTy::Bool => Self::Boolean,
            PayloadTy::Blob(_) => Self::array(Self::Number),
            PayloadTy::Struct(name) | PayloadTy::Enum(name) => Self::Named(name.clone()),
            PayloadTy::HashMap(key, value) | PayloadTy::BTreeMap(key, value) => {
                Self::Map(Box::new(Self::from(&**key)), Box::new(Self::from(&**value)))
            }
            PayloadTy::HashSet(inner) | PayloadTy::BTreeSet(inner) => {
                Self::Set(Box::new(Self::from(&**inner)))
            }
            PayloadTy::Vec(inner) => Self::array(Self::from(&**inner)),
            PayloadTy::Option(inner) => Self::optional_value(Self::from(&**inner)),
            PayloadTy::Tuple(items) => Self::Tuple(items.iter().map(Self::from).collect()),
            PayloadTy::Array(inner, len) => {
                Self::Tuple((0..*len).map(|_| Self::from(&**inner)).collect::<Vec<_>>())
            }
        }
    }
}

impl From<PayloadTy> for Type {
    fn from(value: PayloadTy) -> Self {
        Self::from(&value)
    }
}
