use crate::*;
use brec_scheme::{BlockTy, PayloadTy, SchemeBlockField, SchemeFieldType, SchemePayloadField};

pub const JAVA_PACKAGE: &str = "com.icsmw.brec";
pub const JAVA_PACKAGE_PATH: &str = "com/icsmw/brec";

#[derive(Clone)]
pub struct JavaField {
    pub name: String,
    pub ty: String,
    payload_ty: Option<PayloadTy>,
}

impl JavaField {
    pub fn from_block(field: &SchemeBlockField) -> Result<Self, Error> {
        let SchemeFieldType::Block(ty) = &field.ty else {
            return Err(Error::InvalidScheme(format!(
                "block field {} is not a block field",
                field.name
            )));
        };
        Ok(Self {
            name: field.name.clone(),
            ty: java_block_ty(ty).to_owned(),
            payload_ty: None,
        })
    }

    pub fn from_payload(field: &SchemePayloadField, idx: usize) -> Result<Self, Error> {
        let SchemeFieldType::Payload(ty) = &field.ty else {
            return Err(Error::InvalidScheme(format!(
                "payload field {:?} is not a payload field",
                field.name
            )));
        };
        Ok(Self {
            name: field.name.clone().unwrap_or_else(|| format!("field{idx}")),
            ty: java_payload_ty(ty),
            payload_ty: Some(ty.clone()),
        })
    }

    pub fn from_brec_expr(&self, raw: &str) -> String {
        match &self.payload_ty {
            Some(ty) => payload_from_brec_expr(raw, ty),
            None => format!("({}) {raw}", self.ty),
        }
    }

    pub fn to_brec_expr(&self) -> String {
        match &self.payload_ty {
            Some(ty) => payload_to_brec_expr(&self.name, ty),
            None => self.name.clone(),
        }
    }

    pub fn collect_imports(&self, imports: &mut Vec<&'static str>) {
        if self.ty.contains("BigInteger") {
            imports.push("java.math.BigInteger");
        }
        if self.ty.contains("List<") {
            imports.push("java.util.List");
        }
        if self.ty.contains("Map<") {
            imports.push("java.util.Map");
        }
        if self.ty.contains("Set<") {
            imports.push("java.util.Set");
        }
    }

    pub fn needs_unchecked_cast_suppression(&self) -> bool {
        self.ty.contains('<')
    }

    pub fn needs_array_helpers(&self) -> bool {
        self.ty.ends_with("[]")
    }

    pub fn equals_expr(&self, other: &str) -> String {
        if self.needs_array_helpers() {
            format!("Arrays.equals({}, {}.{})", self.name, other, self.name)
        } else {
            format!("Objects.equals({}, {}.{})", self.name, other, self.name)
        }
    }

    pub fn hash_expr(&self) -> String {
        if self.needs_array_helpers() {
            format!("Arrays.hashCode({})", self.name)
        } else {
            format!("Objects.hashCode({})", self.name)
        }
    }
}

pub fn java_block_ty(ty: &BlockTy) -> &'static str {
    match ty {
        BlockTy::Bool => "Boolean",
        BlockTy::U8
        | BlockTy::U16
        | BlockTy::U32
        | BlockTy::I8
        | BlockTy::I16
        | BlockTy::I32
        | BlockTy::F32
        | BlockTy::LinkedToU8(_) => "Long",
        BlockTy::U64 | BlockTy::U128 | BlockTy::I64 | BlockTy::I128 | BlockTy::F64 => "BigInteger",
        BlockTy::Blob(_) => "byte[]",
    }
}

pub fn java_payload_ty(ty: &PayloadTy) -> String {
    match ty {
        PayloadTy::String => "String".to_owned(),
        PayloadTy::Bool => "Boolean".to_owned(),
        PayloadTy::U8
        | PayloadTy::U16
        | PayloadTy::U32
        | PayloadTy::I8
        | PayloadTy::I16
        | PayloadTy::I32
        | PayloadTy::F32 => "Long".to_owned(),
        PayloadTy::U64 | PayloadTy::U128 | PayloadTy::I64 | PayloadTy::I128 | PayloadTy::F64 => {
            "BigInteger".to_owned()
        }
        PayloadTy::Blob(_) => "byte[]".to_owned(),
        PayloadTy::Struct(name) | PayloadTy::Enum(name) => name.clone(),
        PayloadTy::Vec(inner) | PayloadTy::Array(inner, _) => {
            format!("List<{}>", java_payload_ty(inner))
        }
        PayloadTy::HashSet(inner) | PayloadTy::BTreeSet(inner) => {
            format!("Set<{}>", java_payload_ty(inner))
        }
        PayloadTy::HashMap(key, value) | PayloadTy::BTreeMap(key, value) => {
            format!("Map<{}, {}>", java_payload_ty(key), java_payload_ty(value))
        }
        PayloadTy::Option(inner) => java_payload_ty(inner),
        PayloadTy::Tuple(_) => "List<Object>".to_owned(),
    }
}

pub fn payload_from_brec_expr(raw: &str, ty: &PayloadTy) -> String {
    match ty {
        PayloadTy::Struct(name) | PayloadTy::Enum(name) => format!("{name}.fromBrecObject({raw})"),
        PayloadTy::Vec(inner) | PayloadTy::Array(inner, _) => match inner.as_ref() {
            PayloadTy::Struct(_) | PayloadTy::Enum(_) => {
                format!(
                    "PayloadSupport.mapList({raw}, item -> {})",
                    payload_from_brec_expr("item", inner)
                )
            }
            _ => format!("({}) {raw}", java_payload_ty(ty)),
        },
        PayloadTy::Option(inner) => {
            format!(
                "({raw} == null ? null : {})",
                payload_from_brec_expr(raw, inner)
            )
        }
        _ => format!("({}) {raw}", java_payload_ty(ty)),
    }
}

pub fn payload_to_brec_expr(value: &str, ty: &PayloadTy) -> String {
    match ty {
        PayloadTy::Struct(_) | PayloadTy::Enum(_) => format!("{value}.toBrecObject()"),
        PayloadTy::Vec(inner) | PayloadTy::Array(inner, _) => match inner.as_ref() {
            PayloadTy::Struct(_) | PayloadTy::Enum(_) => {
                let item = format!("(({}) item)", java_payload_ty(inner));
                format!(
                    "PayloadSupport.mapList({value}, item -> {})",
                    payload_to_brec_expr(&item, inner)
                )
            }
            _ => value.to_owned(),
        },
        PayloadTy::Option(inner) => {
            format!(
                "({value} == null ? null : {})",
                payload_to_brec_expr(value, inner)
            )
        }
        _ => value.to_owned(),
    }
}

pub fn lower_camel(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_lowercase().collect::<String>() + chars.as_str()
}

pub fn collect_payload_fields<'a>(
    fields: impl IntoIterator<Item = &'a SchemePayloadField>,
) -> Result<Vec<JavaField>, Error> {
    fields
        .into_iter()
        .enumerate()
        .map(|(idx, field)| JavaField::from_payload(field, idx))
        .collect()
}

pub fn write_args(writer: &mut SourceWriter, fields: &[JavaField]) -> Result<(), Error> {
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            writer.write(", ")?;
        }
        writer.write(format!("{} {}", field.ty, field.name))?;
    }
    Ok(())
}
