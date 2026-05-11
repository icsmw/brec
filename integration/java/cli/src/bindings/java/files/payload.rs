use super::block::write_fields_class_body;
use super::file::{JavaFile, JavaPackage, write_imports};
use crate::*;
use brec_scheme::{SchemeFile, SchemePayload, SchemePayloadVariant, SchemeType};

pub(super) struct PayloadInterfaceFile<'a> {
    model: &'a Model,
}

impl<'a> PayloadInterfaceFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Payload, "Payload.java", |writer| {
            write_imports(writer, &["java.util.Map"])?;
            writer.ln("public interface Payload {")?;
            writer.tab();
            writer.ln("Map<String, Object> toBrecObject();")?;
            writer.ln("")?;
            writer.ln("static Payload fromBrecObject(Object value) {")?;
            writer.tab();
            writer.ln("return PayloadSupport.fromBrecObject(value);")?;
            writer.back();
            writer.ln("}")?;
            writer.back();
            writer.ln("}")
        })
    }
}

pub(super) struct PayloadSupportFile<'a> {
    model: &'a Model,
    scheme: &'a SchemeFile,
}

impl<'a> PayloadSupportFile<'a> {
    pub(super) fn new(model: &'a Model, scheme: &'a SchemeFile) -> Self {
        Self { model, scheme }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(
            self.model,
            JavaPackage::Payload,
            "PayloadSupport.java",
            |writer| {
                write_imports(
                    writer,
                    &[
                        "java.util.ArrayList",
                        "java.util.List",
                        "java.util.Map",
                        "java.util.function.Function",
                    ],
                )?;
                writer.ln("final class PayloadSupport {")?;
                writer.tab();
                writer.ln("private PayloadSupport() {}")?;
                writer.ln("")?;
                writer
                    .ln("static <T> List<T> mapList(Object value, Function<Object, T> mapper) {")?;
                writer.tab();
                writer.ln("ArrayList<T> out = new ArrayList<>();")?;
                writer.ln("for (Object item : (List<?>) value) {")?;
                writer.tab();
                writer.ln("out.add(mapper.apply(item));")?;
                writer.back();
                writer.ln("}")?;
                writer.ln("return out;")?;
                writer.back();
                writer.ln("}")?;
                writer.ln("")?;
                writer.ln("static Payload fromBrecObject(Object value) {")?;
                writer.tab();
                writer.ln("Map<?, ?> map = (Map<?, ?>) value;")?;
                writer.ln("Map.Entry<?, ?> entry = map.entrySet().iterator().next();")?;
                writer.ln("String variant = (String) entry.getKey();")?;
                writer.ln("Object inner = entry.getValue();")?;
                writer.ln("switch (variant) {")?;
                writer.tab();
                for payload in &self.scheme.config.default_payloads {
                    match payload.as_str() {
                        "Bytes" => {
                            writer.ln(r#"case "Bytes": return new Bytes((byte[]) inner);"#)?
                        }
                        "String" => writer
                            .ln(r#"case "String": return new StringPayload((String) inner);"#)?,
                        _ => {}
                    }
                }
                for payload in &self.scheme.payloads {
                    writer.ln(format!(
                        r#"case "{}": return {}.fromBrecObject(inner);"#,
                        payload.fullname, payload.fullname
                    ))?;
                }
                writer.ln(
                r#"default: throw new IllegalArgumentException("unknown payload: " + variant);"#,
            )?;
                writer.back();
                writer.ln("}")?;
                writer.back();
                writer.ln("}")?;
                writer.back();
                writer.ln("}")
            },
        )
    }
}

pub(super) struct DefaultPayloadFile<'a> {
    model: &'a Model,
    class_name: &'static str,
    ty: &'static str,
    variant: &'static str,
}

impl<'a> DefaultPayloadFile<'a> {
    pub(super) fn new(
        model: &'a Model,
        class_name: &'static str,
        ty: &'static str,
        variant: &'static str,
    ) -> Self {
        Self {
            model,
            class_name,
            ty,
            variant,
        }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(
            self.model,
            JavaPackage::Payload,
            format!("{}.java", self.class_name),
            |writer| {
                write_imports(writer, &["java.util.HashMap", "java.util.Map"])?;
                writer.ln(format!(
                    "public final class {} implements Payload {{",
                    self.class_name
                ))?;
                writer.tab();
                writer.ln(format!("public {} value;", self.ty))?;
                writer.ln("")?;
                writer.ln(format!("public {}() {{}}", self.class_name))?;
                writer.ln("")?;
                writer.ln(format!("{}({} value) {{", self.class_name, self.ty))?;
                writer.tab();
                writer.ln("this.value = value;")?;
                writer.back();
                writer.ln("}")?;
                writer.ln("")?;
                writer.ln("public Map<String, Object> toBrecObject() {")?;
                writer.tab();
                writer.ln("HashMap<String, Object> out = new HashMap<>();")?;
                writer.ln(format!(r#"out.put("{}", value);"#, self.variant))?;
                writer.ln("return out;")?;
                writer.back();
                writer.ln("}")?;
                writer.back();
                writer.ln("}")?;
                Ok(())
            },
        )
    }
}

pub(super) struct HelperTypeFile<'a> {
    model: &'a Model,
    ty: &'a SchemeType,
}

impl<'a> HelperTypeFile<'a> {
    pub(super) fn new(model: &'a Model, ty: &'a SchemeType) -> Self {
        Self { model, ty }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        let class_name = self.ty.fullname.clone();
        if self.ty.variants.is_empty() {
            let fields = collect_payload_fields(&self.ty.fields)?;
            JavaFile::new(
                self.model,
                JavaPackage::Payload,
                format!("{class_name}.java"),
                |writer| {
                    write_imports(writer, &payload_imports(&fields))?;
                    writer.ln(format!("public final class {class_name} {{"))?;
                    write_fields_class_body(writer, &class_name, &fields, None)
                },
            )
        } else {
            let variants = self.ty.variants.clone();
            let imports = enum_imports(&variants)?;
            JavaFile::new(
                self.model,
                JavaPackage::Payload,
                format!("{class_name}.java"),
                |writer| {
                    write_imports(writer, &imports)?;
                    write_enum_class(writer, &class_name, &variants, None)
                },
            )
        }
    }
}

pub(super) struct PayloadFile<'a> {
    model: &'a Model,
    payload: &'a SchemePayload,
}

impl<'a> PayloadFile<'a> {
    pub(super) fn new(model: &'a Model, payload: &'a SchemePayload) -> Self {
        Self { model, payload }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        let class_name = self.payload.fullname.clone();
        if self.payload.variants.is_empty() {
            let fields = collect_payload_fields(&self.payload.fields)?;
            JavaFile::new(
                self.model,
                JavaPackage::Payload,
                format!("{class_name}.java"),
                |writer| {
                    write_imports(writer, &payload_imports(&fields))?;
                    writer.ln(format!(
                        "public final class {class_name} implements Payload {{"
                    ))?;
                    write_fields_class_body(writer, &class_name, &fields, Some(&class_name))
                },
            )
        } else {
            let variants = self.payload.variants.clone();
            let imports = enum_imports(&variants)?;
            JavaFile::new(
                self.model,
                JavaPackage::Payload,
                format!("{class_name}.java"),
                |writer| {
                    write_imports(writer, &imports)?;
                    write_enum_class(writer, &class_name, &variants, Some(&class_name))
                },
            )
        }
    }
}

fn write_enum_class(
    writer: &mut SourceWriter,
    class_name: &str,
    variants: &[SchemePayloadVariant],
    variant_wrapper: Option<&str>,
) -> Result<(), Error> {
    let implements = variant_wrapper
        .map(|_| " implements Payload")
        .unwrap_or_default();
    writer.ln(format!("public final class {class_name}{implements} {{"))?;
    writer.tab();
    writer.ln("private final String variant;")?;
    writer.ln("private final Object value;")?;
    writer.ln("")?;
    writer.ln(format!(
        "private {class_name}(String variant, Object value) {{"
    ))?;
    writer.tab();
    writer.ln("this.variant = variant;")?;
    writer.ln("this.value = value;")?;
    writer.back();
    writer.ln("}")?;
    for variant in variants {
        let fields = collect_payload_fields(&variant.fields)?;
        writer.ln("")?;
        writer.write(format!(
            "public static {class_name} {}(",
            lower_camel(&variant.name)
        ))?;
        write_args(writer, &fields)?;
        writer.ln(") {")?;
        writer.tab();
        writer.ln(format!(
            r#"return new {class_name}("{}", {});"#,
            variant.name,
            variant_body_expr(&fields)?
        ))?;
        writer.back();
        writer.ln("}")?;
    }
    writer.ln("")?;
    if needs_enum_unchecked_cast_suppression(variants)? {
        writer.ln(r#"@SuppressWarnings("unchecked")"#)?;
    }
    writer.ln(format!(
        "static {class_name} fromBrecObject(Object value) {{"
    ))?;
    writer.tab();
    writer.ln("Map<?, ?> map = (Map<?, ?>) value;")?;
    writer.ln("Map.Entry<?, ?> entry = map.entrySet().iterator().next();")?;
    writer.ln(format!(
        "return new {class_name}((String) entry.getKey(), entry.getValue());"
    ))?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("public Map<String, Object> toBrecObject() {")?;
    writer.tab();
    writer.ln("HashMap<String, Object> body = new HashMap<>();")?;
    writer.ln("body.put(variant, value);")?;
    if let Some(variant) = variant_wrapper {
        writer.ln("HashMap<String, Object> out = new HashMap<>();")?;
        writer.ln(format!(r#"out.put("{variant}", body);"#))?;
        writer.ln("return out;")?;
    } else {
        writer.ln("return body;")?;
    }
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    Ok(())
}

fn payload_imports(fields: &[JavaField]) -> Vec<&'static str> {
    let mut imports = vec!["java.util.HashMap", "java.util.Map"];
    for field in fields {
        field.collect_imports(&mut imports);
    }
    imports
}

fn enum_imports(variants: &[SchemePayloadVariant]) -> Result<Vec<&'static str>, Error> {
    let mut imports = vec!["java.util.HashMap", "java.util.Map"];
    for variant in variants {
        for field in collect_payload_fields(&variant.fields)? {
            field.collect_imports(&mut imports);
        }
    }
    Ok(imports)
}

fn needs_enum_unchecked_cast_suppression(variants: &[SchemePayloadVariant]) -> Result<bool, Error> {
    for variant in variants {
        if collect_payload_fields(&variant.fields)?
            .iter()
            .any(JavaField::needs_unchecked_cast_suppression)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn variant_body_expr(fields: &[JavaField]) -> Result<String, Error> {
    if fields.is_empty() {
        return Ok("null".to_owned());
    }
    if fields.len() == 1 && fields[0].name == "field0" {
        return Ok(fields[0].to_brec_expr());
    }
    Ok("null".to_owned())
}
