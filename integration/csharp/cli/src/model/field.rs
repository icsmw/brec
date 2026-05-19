use super::CSharpType;
use crate::{Error, SourceWriter};

#[derive(Clone)]
pub struct FieldDef {
    pub key: String,
    pub name: String,
    pub ty: CSharpType,
    pub nullable: bool,
}

pub(super) fn write_class(
    writer: &mut SourceWriter,
    name: &str,
    parent: Option<&str>,
    fields: &[FieldDef],
) -> Result<(), Error> {
    let inherits = parent
        .map(|parent| format!(" : {parent}"))
        .unwrap_or_default();
    writer.ln(format!("public sealed class {name}{inherits}"))?;
    writer.ln("{")?;
    writer.tab();
    for field in fields {
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
    writer.write(format!("public {name}("))?;
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
    for field in fields {
        writer.ln(format!(
            "{} = {};",
            field.name,
            constructor_arg_name(&field.name)
        ))?;
    }
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    write_from_native(writer, name, fields)?;
    writer.ln("")?;
    write_to_native(writer, fields, parent.is_some())?;
    writer.back();
    writer.ln("}")
}

pub(super) fn constructor_arg_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
        None => "value".to_owned(),
    }
}

fn write_from_native(
    writer: &mut SourceWriter,
    name: &str,
    fields: &[FieldDef],
) -> Result<(), Error> {
    writer.ln(format!(
        "internal static {name} FromNativeObject(ValueHandle handle)"
    ))?;
    writer.ln("{")?;
    writer.tab();
    for field in fields {
        writer.ln(format!(
            "using var {}Value = NativeValue.GetField(handle, \"{}\");",
            constructor_arg_name(&field.name),
            field.key
        ))?;
        let expr = if field.nullable {
            format!(
                "NativeValue.Kind({}Value) == NativeValueKind.Null ? null : {}",
                constructor_arg_name(&field.name),
                field.ty.from_native_nullable_expr(&format!(
                    "{}Value",
                    constructor_arg_name(&field.name)
                ))
            )
        } else {
            field
                .ty
                .from_native_expr(&format!("{}Value", constructor_arg_name(&field.name)))
        };
        writer.ln(format!(
            "var {} = {};",
            constructor_arg_name(&field.name),
            expr
        ))?;
    }
    writer.ln(format!(
        "return new {name}({});",
        fields
            .iter()
            .map(|field| constructor_arg_name(&field.name))
            .collect::<Vec<_>>()
            .join(", ")
    ))?;
    writer.back();
    writer.ln("}")
}

fn write_to_native(
    writer: &mut SourceWriter,
    fields: &[FieldDef],
    is_override: bool,
) -> Result<(), Error> {
    writer.ln(format!(
        "internal {}ValueHandle ToNativeObject()",
        if is_override { "override " } else { "" }
    ))?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("var obj = NativeValue.NewObject();")?;
    writer.ln("try")?;
    writer.ln("{")?;
    writer.tab();
    for field in fields {
        let expr = if field.nullable {
            field.ty.to_native_nullable_expr(&field.name)
        } else {
            field.ty.to_native_expr(&field.name)
        };
        writer.ln(format!("using (var value = {expr})"))?;
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
    writer.back();
    writer.ln("}")
}

pub(super) fn write_tuple_record(
    writer: &mut SourceWriter,
    name: &str,
    parent: Option<&str>,
    items: &[CSharpType],
) -> Result<(), Error> {
    let fields = items
        .iter()
        .enumerate()
        .map(|(idx, ty)| FieldDef {
            key: idx.to_string(),
            name: format!("Item{}", idx + 1),
            ty: ty.clone(),
            nullable: false,
        })
        .collect::<Vec<_>>();
    write_class(writer, name, parent, &fields)
}
