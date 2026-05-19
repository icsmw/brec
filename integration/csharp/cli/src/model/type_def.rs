use super::CSharpType;
use super::field::{FieldDef, constructor_arg_name, write_class, write_tuple_record};
use super::ident::csharp_type_name;
use super::names::TypeNames;
use super::resolver::{named_field, payload_field_type};
use crate::{Error, SourceWritable};
use brec_scheme::{SchemePayload, SchemePayloadField, SchemePayloadVariant, SchemeType};

pub struct TypeDef {
    pub key: String,
    pub name: String,
    pub body: TypeBody,
}

pub enum TypeBody {
    Empty,
    Struct(Vec<FieldDef>),
    Tuple(Vec<CSharpType>),
    Enum(Vec<VariantDef>),
}

pub struct VariantDef {
    pub key: String,
    pub name: String,
    pub body: VariantBody,
}

pub enum VariantBody {
    Unit,
    Named(Vec<FieldDef>),
    Single(CSharpType),
    Tuple(Vec<CSharpType>),
}

impl TypeDef {
    pub(super) fn from_payload(payload: &SchemePayload, names: &TypeNames) -> Result<Self, Error> {
        Self::from_parts(&payload.fullname, &payload.fields, &payload.variants, names)
    }

    pub(super) fn from_scheme_type(ty: &SchemeType, names: &TypeNames) -> Result<Self, Error> {
        Self::from_parts(&ty.fullname, &ty.fields, &ty.variants, names)
    }

    fn from_parts(
        name: &str,
        fields: &[SchemePayloadField],
        variants: &[SchemePayloadVariant],
        names: &TypeNames,
    ) -> Result<Self, Error> {
        let body = match (fields.is_empty(), variants.is_empty()) {
            (true, true) => TypeBody::Empty,
            (false, true) => TypeBody::from_fields(name, fields, names)?,
            (true, false) => TypeBody::Enum(
                variants
                    .iter()
                    .map(|variant| VariantDef::from_scheme(name, variant, names))
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
            key: name.to_owned(),
            name: csharp_type_name(name),
            body,
        })
    }

    pub(crate) fn write_with_parent(
        &self,
        writer: &mut crate::SourceWriter,
        parent: &str,
    ) -> Result<(), Error> {
        match &self.body {
            TypeBody::Empty => write_class(writer, &self.name, Some(parent), &[]),
            TypeBody::Struct(fields) => write_class(writer, &self.name, Some(parent), fields),
            TypeBody::Tuple(items) => write_tuple_record(writer, &self.name, Some(parent), items),
            TypeBody::Enum(variants) => self.write_enum(writer, Some(parent), variants),
        }
    }

    fn write_enum(
        &self,
        writer: &mut crate::SourceWriter,
        parent: Option<&str>,
        variants: &[VariantDef],
    ) -> Result<(), Error> {
        let inherits = parent
            .map(|parent| format!(" : {parent}"))
            .unwrap_or_default();
        writer.ln(format!("public abstract class {}{}", self.name, inherits))?;
        writer.ln("{")?;
        writer.tab();
        writer.ln(format!("private protected {}() {{ }}", self.name))?;
        writer.ln("")?;
        writer.ln("public enum Kind")?;
        writer.ln("{")?;
        writer.tab();
        for variant in variants {
            writer.ln(format!("{},", variant.name))?;
        }
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("public abstract Kind Variant { get; }")?;
        writer.ln("private protected abstract string NativeKey { get; }")?;
        writer.ln("internal abstract ValueHandle ToNativeElement();")?;
        writer.ln(format!(
            "internal {}ValueHandle ToNativeObject()",
            if parent.is_some() { "override " } else { "" }
        ))?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("var obj = NativeValue.NewObject();")?;
        writer.ln("try")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("using var value = ToNativeElement();")?;
        writer.ln("NativeValue.PutField(obj, NativeKey, value);")?;
        writer.ln("return obj;")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("catch")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("obj.Dispose();")?;
        writer.ln("throw;")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln(format!(
            "internal static {} FromNativeObject(ValueHandle handle)",
            self.name
        ))?;
        writer.ln("{")?;
        writer.tab();
        for variant in variants {
            writer.ln(format!(
                "if (NativeValue.HasField(handle, \"{}\"))",
                variant.key
            ))?;
            writer.ln("{")?;
            writer.tab();
            writer.ln(format!(
                "using var inner = NativeValue.GetField(handle, \"{}\");",
                variant.key
            ))?;
            writer.ln(format!(
                "return Element{}.FromNativeElement(inner);",
                variant.name
            ))?;
            writer.back();
            writer.ln("}")?;
        }
        writer.ln(format!(
            "throw new InvalidOperationException(\"Unknown {} variant\");",
            self.name
        ))?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for variant in variants {
            variant.write_nested_class(writer, &self.name)?;
            writer.ln("")?;
        }
        writer.back();
        writer.ln("}")
    }
}

impl TypeBody {
    fn from_fields(
        owner: &str,
        fields: &[SchemePayloadField],
        names: &TypeNames,
    ) -> Result<Self, Error> {
        if fields.iter().all(|field| field.name.is_some()) {
            return Ok(Self::Struct(
                fields
                    .iter()
                    .map(|field| named_field(owner, field, names))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        if fields.iter().all(|field| field.name.is_none()) {
            return Ok(Self::Tuple(
                fields
                    .iter()
                    .map(|field| payload_field_type(owner, &field.ty, names).map(|(ty, _)| ty))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }

        Err(Error::InvalidScheme(format!(
            "type {} mixes named and unnamed fields",
            owner
        )))
    }
}

impl VariantDef {
    fn from_scheme(
        owner: &str,
        variant: &SchemePayloadVariant,
        names: &TypeNames,
    ) -> Result<Self, Error> {
        Ok(Self {
            key: variant.name.clone(),
            name: csharp_type_name(&variant.name),
            body: VariantBody::from_fields(owner, &variant.name, &variant.fields, names)?,
        })
    }

    fn write_nested_class(
        &self,
        writer: &mut crate::SourceWriter,
        parent: &str,
    ) -> Result<(), Error> {
        let class_name = format!("Element{}", self.name);
        let fields = self.fields();
        writer.ln(format!("public sealed class {class_name} : {parent}"))?;
        writer.ln("{")?;
        writer.tab();
        writer.ln(format!(
            "public override Kind Variant => Kind.{};",
            self.name
        ))?;
        writer.ln(format!(
            "private protected override string NativeKey => \"{}\";",
            self.key
        ))?;
        for field in &fields {
            writer.ln(format!(
                "public {}{} {} {{ get; }}",
                field.ty.write_ref(),
                if field.nullable { "?" } else { "" },
                field.name
            ))?;
        }
        if !fields.is_empty() {
            writer.ln("")?;
        }
        writer.write(format!("public {class_name}("))?;
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                writer.write(", ")?;
            }
            writer.write(format!(
                "{}{} {}",
                field.ty.write_ref(),
                if field.nullable { "?" } else { "" },
                constructor_arg_name(&field.name)
            ))?;
        }
        writer.ln(")")?;
        writer.ln("{")?;
        writer.tab();
        for field in &fields {
            writer.ln(format!(
                "{} = {};",
                field.name,
                constructor_arg_name(&field.name)
            ))?;
        }
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        self.write_variant_from_native(writer, &class_name, &fields)?;
        writer.ln("")?;
        self.write_variant_to_native(writer, &fields)?;
        writer.back();
        writer.ln("}")
    }

    fn fields(&self) -> Vec<FieldDef> {
        match &self.body {
            VariantBody::Unit => Vec::new(),
            VariantBody::Named(fields) => fields.clone(),
            VariantBody::Single(ty) => vec![FieldDef {
                key: "Value".to_owned(),
                name: "Value".to_owned(),
                ty: ty.clone(),
                nullable: false,
            }],
            VariantBody::Tuple(items) => items
                .iter()
                .enumerate()
                .map(|(idx, ty)| FieldDef {
                    key: idx.to_string(),
                    name: format!("Item{}", idx + 1),
                    ty: ty.clone(),
                    nullable: false,
                })
                .collect(),
        }
    }

    fn write_variant_from_native(
        &self,
        writer: &mut crate::SourceWriter,
        class_name: &str,
        fields: &[FieldDef],
    ) -> Result<(), Error> {
        writer.ln(format!(
            "internal static {class_name} FromNativeElement(ValueHandle handle)"
        ))?;
        writer.ln("{")?;
        writer.tab();
        match &self.body {
            VariantBody::Unit => {}
            VariantBody::Single(ty) => {
                writer.ln(format!("var value = {};", ty.from_native_expr("handle")))?;
            }
            VariantBody::Named(_) => {
                for field in fields {
                    writer.ln(format!(
                        "using var {}Value = NativeValue.GetField(handle, \"{}\");",
                        constructor_arg_name(&field.name),
                        field.key
                    ))?;
                    writer.ln(format!(
                        "var {} = {};",
                        constructor_arg_name(&field.name),
                        field.ty.from_native_expr(&format!(
                            "{}Value",
                            constructor_arg_name(&field.name)
                        ))
                    ))?;
                }
            }
            VariantBody::Tuple(_) => {
                for field in fields {
                    writer.ln(format!(
                        "using var {}Value = NativeValue.GetField(handle, \"{}\");",
                        constructor_arg_name(&field.name),
                        field.key
                    ))?;
                    writer.ln(format!(
                        "var {} = {};",
                        constructor_arg_name(&field.name),
                        field.ty.from_native_expr(&format!(
                            "{}Value",
                            constructor_arg_name(&field.name)
                        ))
                    ))?;
                }
            }
        }
        writer.ln(format!(
            "return new {class_name}({});",
            fields
                .iter()
                .map(|field| constructor_arg_name(&field.name))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        writer.back();
        writer.ln("}")
    }

    fn write_variant_to_native(
        &self,
        writer: &mut crate::SourceWriter,
        fields: &[FieldDef],
    ) -> Result<(), Error> {
        writer.ln("internal override ValueHandle ToNativeElement()")?;
        writer.ln("{")?;
        writer.tab();
        match &self.body {
            VariantBody::Unit => writer.ln("return NativeValue.Null();")?,
            VariantBody::Single(ty) => {
                writer.ln(format!("return {};", ty.to_native_expr("Value")))?
            }
            VariantBody::Named(_) | VariantBody::Tuple(_) => {
                writer.ln("var obj = NativeValue.NewObject();")?;
                writer.ln("try")?;
                writer.ln("{")?;
                writer.tab();
                for field in fields {
                    writer.ln(format!(
                        "using (var value = {})",
                        field.ty.to_native_expr(&field.name)
                    ))?;
                    writer.ln("{")?;
                    writer.tab();
                    writer.ln(format!(
                        "NativeValue.PutField(obj, \"{}\", value);",
                        field.key
                    ))?;
                    writer.back();
                    writer.ln("}")?;
                }
                writer.ln("return obj;")?;
                writer.back();
                writer.ln("}")?;
                writer.ln("catch")?;
                writer.ln("{")?;
                writer.tab();
                writer.ln("obj.Dispose();")?;
                writer.ln("throw;")?;
                writer.back();
                writer.ln("}")?;
            }
        }
        writer.back();
        writer.ln("}")
    }
}

impl VariantBody {
    fn from_fields(
        owner: &str,
        variant: &str,
        fields: &[SchemePayloadField],
        names: &TypeNames,
    ) -> Result<Self, Error> {
        if fields.is_empty() {
            return Ok(Self::Unit);
        }
        if fields.iter().all(|field| field.name.is_some()) {
            return Ok(Self::Named(
                fields
                    .iter()
                    .map(|field| named_field(owner, field, names))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }
        if fields.iter().all(|field| field.name.is_none()) {
            let items = fields
                .iter()
                .map(|field| payload_field_type(owner, &field.ty, names).map(|(ty, _)| ty))
                .collect::<Result<Vec<_>, _>>()?;
            if items.len() == 1 {
                return Ok(Self::Single(items.into_iter().next().expect("one item")));
            }
            return Ok(Self::Tuple(items));
        }

        Err(Error::InvalidScheme(format!(
            "payload enum variant {}::{} mixes named and unnamed fields",
            owner, variant
        )))
    }
}

impl SourceWritable for TypeDef {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        match &self.body {
            TypeBody::Empty => write_class(writer, &self.name, None, &[]),
            TypeBody::Struct(fields) => write_class(writer, &self.name, None, fields),
            TypeBody::Tuple(items) => write_tuple_record(writer, &self.name, None, items),
            TypeBody::Enum(variants) => self.write_enum(writer, None, variants),
        }
    }
}
