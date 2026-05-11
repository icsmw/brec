use crate::Error;
use brec_scheme::{PayloadTy, SchemeFieldType, SchemeFile};
use std::collections::{BTreeSet, HashMap};

/// Registry of scheme type names that can be referenced from payload fields.
///
/// Rust macros can report short names, full names, or module paths depending on
/// where a type appears. `TypeNames` normalizes those spellings and also
/// validates that every named reference has been exported into `scheme.types`.
pub(super) struct TypeNames;

impl TypeNames {
    fn known_names(scheme: &SchemeFile) -> HashMap<String, String> {
        let mut by_raw = HashMap::new();

        for payload in &scheme.payloads {
            Self::register(
                &mut by_raw,
                &payload.name,
                &payload.fullname,
                &payload.fullpath,
            );
        }

        for scheme_type in &scheme.types {
            Self::register(
                &mut by_raw,
                &scheme_type.name,
                &scheme_type.fullname,
                &scheme_type.fullpath,
            );
        }

        by_raw
    }

    /// Collects named payload references that must be present in `scheme.types`.
    ///
    /// Without this validation a missing `#[payload(include)]` would produce a
    /// Java reference to a type that is never declared.
    fn referenced_names(scheme: &SchemeFile) -> Vec<String> {
        let mut refs = BTreeSet::new();

        for payload in &scheme.payloads {
            for field in &payload.fields {
                Self::collect_field_refs(&field.ty, &mut refs);
            }
            for variant in &payload.variants {
                for field in &variant.fields {
                    Self::collect_field_refs(&field.ty, &mut refs);
                }
            }
        }

        for scheme_type in &scheme.types {
            for field in &scheme_type.fields {
                Self::collect_field_refs(&field.ty, &mut refs);
            }
            for variant in &scheme_type.variants {
                for field in &variant.fields {
                    Self::collect_field_refs(&field.ty, &mut refs);
                }
            }
        }

        refs.into_iter().collect()
    }

    fn register(by_raw: &mut HashMap<String, String>, name: &str, fullname: &str, fullpath: &str) {
        by_raw.insert(name.to_owned(), fullname.to_owned());
        by_raw.insert(fullname.to_owned(), fullname.to_owned());
        by_raw.insert(fullpath.to_owned(), fullname.to_owned());
        by_raw.insert(Self::normalize(fullpath), fullname.to_owned());
    }

    fn collect_field_refs(ty: &SchemeFieldType, refs: &mut BTreeSet<String>) {
        if let SchemeFieldType::Payload(ty) = ty {
            Self::collect_payload_refs(ty, refs);
        }
    }

    fn collect_payload_refs(ty: &PayloadTy, refs: &mut BTreeSet<String>) {
        match ty {
            PayloadTy::Struct(name) | PayloadTy::Enum(name) => {
                refs.insert(name.clone());
            }
            PayloadTy::HashMap(key, value) | PayloadTy::BTreeMap(key, value) => {
                Self::collect_payload_refs(key, refs);
                Self::collect_payload_refs(value, refs);
            }
            PayloadTy::HashSet(inner)
            | PayloadTy::BTreeSet(inner)
            | PayloadTy::Vec(inner)
            | PayloadTy::Option(inner) => Self::collect_payload_refs(inner, refs),
            PayloadTy::Tuple(items) => {
                for item in items {
                    Self::collect_payload_refs(item, refs);
                }
            }
            PayloadTy::Array(inner, _) => Self::collect_payload_refs(inner, refs),
            PayloadTy::String
            | PayloadTy::U8
            | PayloadTy::U16
            | PayloadTy::U32
            | PayloadTy::U64
            | PayloadTy::U128
            | PayloadTy::I8
            | PayloadTy::I16
            | PayloadTy::I32
            | PayloadTy::I64
            | PayloadTy::I128
            | PayloadTy::F32
            | PayloadTy::F64
            | PayloadTy::Bool
            | PayloadTy::Blob(_) => {}
        }
    }

    fn normalize(raw: &str) -> String {
        raw.split("::")
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl TryFrom<&SchemeFile> for TypeNames {
    type Error = Error;

    fn try_from(scheme: &SchemeFile) -> Result<Self, Self::Error> {
        let referenced = Self::referenced_names(scheme);
        let by_raw = Self::known_names(scheme);
        let missing = referenced
            .into_iter()
            .filter(|raw| {
                let normalized = Self::normalize(raw);
                !by_raw.contains_key(raw) && !by_raw.contains_key(&normalized)
            })
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(Error::MissingIncludedTypes(missing));
        }

        Ok(Self)
    }
}
