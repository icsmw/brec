use super::ident::csharp_type_name;
use super::names::TypeNames;
use brec_scheme::{BlockTy, PayloadTy};

#[derive(Clone)]
pub enum CSharpType {
    Named(String),
    Primitive(&'static str),
    Bytes,
    List(Box<CSharpType>),
    Dictionary(Box<CSharpType>, Box<CSharpType>),
    Set(Box<CSharpType>),
    Array(Box<CSharpType>),
    Tuple(Vec<CSharpType>),
}

impl CSharpType {
    pub fn write_ref(&self) -> String {
        match self {
            Self::Named(name) => csharp_type_name(name),
            Self::Primitive(name) => (*name).to_owned(),
            Self::Bytes => "byte[]".to_owned(),
            Self::List(inner) => format!("IReadOnlyList<{}>", inner.write_ref()),
            Self::Dictionary(key, value) => {
                format!(
                    "IReadOnlyDictionary<{}, {}>",
                    key.write_ref(),
                    value.write_ref()
                )
            }
            Self::Set(inner) => format!("IReadOnlySet<{}>", inner.write_ref()),
            Self::Array(inner) => format!("{}[]", inner.write_ref()),
            Self::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .enumerate()
                    .map(|(idx, ty)| format!("{} Item{}", ty.write_ref(), idx + 1))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    pub fn from_native_expr(&self, handle: &str) -> String {
        match self {
            Self::Named(name) => format!("{}.FromNativeObject({handle})", csharp_type_name(name)),
            Self::Primitive("byte") => format!("NativeValue.AsByte({handle})"),
            Self::Primitive("ushort") => format!("NativeValue.AsUInt16({handle})"),
            Self::Primitive("uint") => format!("NativeValue.AsUInt32({handle})"),
            Self::Primitive("ulong") => format!("NativeValue.AsUInt64({handle})"),
            Self::Primitive("UInt128") => format!("NativeValue.AsUInt128({handle})"),
            Self::Primitive("sbyte") => format!("NativeValue.AsSByte({handle})"),
            Self::Primitive("short") => format!("NativeValue.AsInt16({handle})"),
            Self::Primitive("int") => format!("NativeValue.AsInt32({handle})"),
            Self::Primitive("long") => format!("NativeValue.AsInt64({handle})"),
            Self::Primitive("Int128") => format!("NativeValue.AsInt128({handle})"),
            Self::Primitive("float") => format!("NativeValue.AsSingle({handle})"),
            Self::Primitive("double") => format!("NativeValue.AsDouble({handle})"),
            Self::Primitive("bool") => format!("NativeValue.AsBoolean({handle})"),
            Self::Primitive("string") => format!("NativeValue.AsString({handle})"),
            Self::Primitive(other) => format!("NativeValue.Unsupported<{other}>({handle})"),
            Self::Bytes => format!("NativeValue.AsBytes({handle})"),
            Self::List(inner) => format!(
                "NativeValue.AsList({}, static item => {})",
                handle,
                inner.from_native_expr("item")
            ),
            Self::Array(inner) => format!(
                "NativeValue.AsList({}, static item => {}).ToArray()",
                handle,
                inner.from_native_expr("item")
            ),
            Self::Set(inner) => format!(
                "NativeValue.AsHashSet({}, static item => {})",
                handle,
                inner.from_native_expr("item")
            ),
            Self::Dictionary(_, _) => format!("NativeValue.UnsupportedDictionary({handle})"),
            Self::Tuple(_) => format!("NativeValue.UnsupportedTuple({handle})"),
        }
    }

    pub fn from_native_nullable_expr(&self, handle: &str) -> String {
        let value = self.from_native_expr(handle);
        if self.is_value_type() {
            format!("({}?){}", self.write_ref(), value)
        } else {
            value
        }
    }

    pub fn to_native_expr(&self, value: &str) -> String {
        match self {
            Self::Named(_) => format!("{value}.ToNativeObject()"),
            Self::Primitive("byte") => format!("NativeValue.FromByte({value})"),
            Self::Primitive("ushort") => format!("NativeValue.FromUInt16({value})"),
            Self::Primitive("uint") => format!("NativeValue.FromUInt32({value})"),
            Self::Primitive("ulong") => format!("NativeValue.FromUInt64({value})"),
            Self::Primitive("UInt128") => format!("NativeValue.FromUInt128({value})"),
            Self::Primitive("sbyte") => format!("NativeValue.FromSByte({value})"),
            Self::Primitive("short") => format!("NativeValue.FromInt16({value})"),
            Self::Primitive("int") => format!("NativeValue.FromInt32({value})"),
            Self::Primitive("long") => format!("NativeValue.FromInt64({value})"),
            Self::Primitive("Int128") => format!("NativeValue.FromInt128({value})"),
            Self::Primitive("float") => format!("NativeValue.FromSingle({value})"),
            Self::Primitive("double") => format!("NativeValue.FromDouble({value})"),
            Self::Primitive("bool") => format!("NativeValue.FromBoolean({value})"),
            Self::Primitive("string") => format!("NativeValue.FromString({value})"),
            Self::Primitive(other) => format!("NativeValue.Unsupported<{other}>({value})"),
            Self::Bytes => format!("NativeValue.FromBytes({value})"),
            Self::List(inner) | Self::Array(inner) | Self::Set(inner) => format!(
                "NativeValue.FromList({}, static item => {})",
                value,
                inner.to_native_expr("item")
            ),
            Self::Dictionary(_, _) => format!("NativeValue.UnsupportedDictionary({value})"),
            Self::Tuple(_) => format!("NativeValue.UnsupportedTuple({value})"),
        }
    }

    pub fn to_native_nullable_expr(&self, value: &str) -> String {
        let inner = if self.is_value_type() {
            format!("{value}.Value")
        } else {
            value.to_owned()
        };
        format!(
            "{value} is null ? NativeValue.Null() : {}",
            self.to_native_expr(&inner)
        )
    }

    fn is_value_type(&self) -> bool {
        matches!(
            self,
            Self::Primitive("byte")
                | Self::Primitive("ushort")
                | Self::Primitive("uint")
                | Self::Primitive("ulong")
                | Self::Primitive("UInt128")
                | Self::Primitive("sbyte")
                | Self::Primitive("short")
                | Self::Primitive("int")
                | Self::Primitive("long")
                | Self::Primitive("Int128")
                | Self::Primitive("float")
                | Self::Primitive("double")
                | Self::Primitive("bool")
        )
    }

    pub(super) fn from_block_ty(ty: &BlockTy) -> Self {
        match ty {
            BlockTy::U8 | BlockTy::LinkedToU8(_) => Self::Primitive("byte"),
            BlockTy::U16 => Self::Primitive("ushort"),
            BlockTy::U32 => Self::Primitive("uint"),
            BlockTy::U64 => Self::Primitive("ulong"),
            BlockTy::U128 => Self::Primitive("UInt128"),
            BlockTy::I8 => Self::Primitive("sbyte"),
            BlockTy::I16 => Self::Primitive("short"),
            BlockTy::I32 => Self::Primitive("int"),
            BlockTy::I64 => Self::Primitive("long"),
            BlockTy::I128 => Self::Primitive("Int128"),
            BlockTy::F32 => Self::Primitive("float"),
            BlockTy::F64 => Self::Primitive("double"),
            BlockTy::Bool => Self::Primitive("bool"),
            BlockTy::Blob(_) => Self::Bytes,
        }
    }

    pub(super) fn from_payload_ty(ty: &PayloadTy, names: &TypeNames) -> (Self, bool) {
        match ty {
            PayloadTy::String => (Self::Primitive("string"), false),
            PayloadTy::U8 => (Self::Primitive("byte"), false),
            PayloadTy::U16 => (Self::Primitive("ushort"), false),
            PayloadTy::U32 => (Self::Primitive("uint"), false),
            PayloadTy::U64 => (Self::Primitive("ulong"), false),
            PayloadTy::U128 => (Self::Primitive("UInt128"), false),
            PayloadTy::I8 => (Self::Primitive("sbyte"), false),
            PayloadTy::I16 => (Self::Primitive("short"), false),
            PayloadTy::I32 => (Self::Primitive("int"), false),
            PayloadTy::I64 => (Self::Primitive("long"), false),
            PayloadTy::I128 => (Self::Primitive("Int128"), false),
            PayloadTy::F32 => (Self::Primitive("float"), false),
            PayloadTy::F64 => (Self::Primitive("double"), false),
            PayloadTy::Bool => (Self::Primitive("bool"), false),
            PayloadTy::Blob(_) => (Self::Bytes, false),
            PayloadTy::Struct(name) | PayloadTy::Enum(name) => {
                (Self::Named(names.resolve(name).to_owned()), false)
            }
            PayloadTy::HashMap(key, value) | PayloadTy::BTreeMap(key, value) => (
                Self::Dictionary(
                    Box::new(Self::from_payload_ty(key, names).0),
                    Box::new(Self::from_payload_ty(value, names).0),
                ),
                false,
            ),
            PayloadTy::HashSet(inner) | PayloadTy::BTreeSet(inner) => (
                Self::Set(Box::new(Self::from_payload_ty(inner, names).0)),
                false,
            ),
            PayloadTy::Vec(inner) => (
                Self::List(Box::new(Self::from_payload_ty(inner, names).0)),
                false,
            ),
            PayloadTy::Option(inner) => {
                let (inner, _) = Self::from_payload_ty(inner, names);
                (inner, true)
            }
            PayloadTy::Tuple(items) => (
                Self::Tuple(
                    items
                        .iter()
                        .map(|item| Self::from_payload_ty(item, names).0)
                        .collect(),
                ),
                false,
            ),
            PayloadTy::Array(inner, _) => (
                Self::Array(Box::new(Self::from_payload_ty(inner, names).0)),
                false,
            ),
        }
    }
}
