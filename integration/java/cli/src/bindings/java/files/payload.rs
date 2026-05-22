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
            writer.block(
                r#"
public interface Payload {
	Map<String, Object> toBrecObject();

	static Payload fromBrecObject(Object value) {
		return PayloadSupport.fromBrecObject(value);
	}
}
"#,
            )
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
                writer.block(
                    r#"
final class PayloadSupport {
	private PayloadSupport() {}

	static <T> List<T> mapList(Object value, Function<Object, T> mapper) {
		List<?> source = (List<?>) value;
		ArrayList<T> out = new ArrayList<>(source.size());
		for (Object item : source) {
			out.add(mapper.apply(item));
		}
		return out;
	}

	static Payload fromBrecObject(Object value) {
		Map<?, ?> map = (Map<?, ?>) value;
		Map.Entry<?, ?> entry = map.entrySet().iterator().next();
		String variant = (String) entry.getKey();
		Object inner = entry.getValue();
		switch (variant) {
"#,
                )?;
                for payload in &self.scheme.config.default_payloads {
                    match payload.as_str() {
                        "Bytes" => {
                            writer.ln("\t\t\tcase \"Bytes\": return new Bytes((byte[]) inner);")?
                        }
                        "String" => writer.ln(
                            "\t\t\tcase \"String\": return new StringPayload((String) inner);",
                        )?,
                        _ => {}
                    }
                }
                for payload in &self.scheme.payloads {
                    writer.ln(format!(
                        "\t\t\tcase \"{}\": return {}.fromBrecObject(inner);",
                        payload.fullname, payload.fullname
                    ))?;
                }
                writer.ln(
                "\t\t\tdefault: throw new IllegalArgumentException(\"unknown payload: \" + variant);",
            )?;
                writer.block(
                    r#"
		}
	}
}
"#,
                )
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
                let mut imports = vec!["java.util.HashMap", "java.util.Map", "java.util.Objects"];
                if self.ty.ends_with("[]") {
                    imports.push("java.util.Arrays");
                }
                write_imports(writer, &imports)?;
                writer.block(format!(
                    r#"
public final class {} implements Payload {{
	public {} value;

	public {}() {{}}

	{}({} value) {{
		this.value = value;
	}}

	public Map<String, Object> toBrecObject() {{
		HashMap<String, Object> out = new HashMap<>(1);
		out.put("{}", value);
		return out;
	}}

	@Override
	public boolean equals(Object other) {{
		if (!(other instanceof {})) {{
			return false;
		}}
		{} that = ({}) other;
"#,
                    self.class_name,
                    self.ty,
                    self.class_name,
                    self.class_name,
                    self.ty,
                    self.variant,
                    self.class_name,
                    self.class_name,
                    self.class_name
                ))?;
                if self.ty.ends_with("[]") {
                    writer.ln("\t\treturn Arrays.equals(value, that.value);")?;
                } else {
                    writer.ln("\t\treturn Objects.equals(value, that.value);")?;
                }
                writer.block(
                    r#"
	}

	@Override
	public int hashCode() {
"#,
                )?;
                if self.ty.ends_with("[]") {
                    writer.ln("\t\treturn Arrays.hashCode(value);")?;
                } else {
                    writer.ln("\t\treturn Objects.hashCode(value);")?;
                }
                writer.block(
                    r#"
	}
}
"#,
                )
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
    writer.block(format!(
        r#"
public final class {class_name}{implements} {{
	private final String variant;
	private final Object value;

	private {class_name}(String variant, Object value) {{
		this.variant = variant;
		this.value = value;
	}}
"#
    ))?;
    for variant in variants {
        let fields = collect_payload_fields(&variant.fields)?;
        writer.ln("")?;
        writer.write(format!(
            "\tpublic static {class_name} {}(",
            lower_camel(&variant.name)
        ))?;
        write_args(writer, &fields)?;
        writer.ln(") {")?;
        writer.ln(format!(
            "\t\treturn new {class_name}(\"{}\", {});",
            variant.name,
            variant_body_expr(&fields)?
        ))?;
        writer.ln("\t}")?;
    }
    writer.ln("")?;
    if needs_enum_unchecked_cast_suppression(variants)? {
        writer.ln("\t@SuppressWarnings(\"unchecked\")")?;
    }
    writer.ln(format!(
        "\tstatic {class_name} fromBrecObject(Object value) {{"
    ))?;
    writer.ln("\t\tMap<?, ?> map = (Map<?, ?>) value;")?;
    writer.ln("\t\tMap.Entry<?, ?> entry = map.entrySet().iterator().next();")?;
    writer.ln(format!(
        "\t\treturn new {class_name}((String) entry.getKey(), entry.getValue());"
    ))?;
    writer.ln("\t}")?;
    writer.ln("")?;
    writer.ln("\tpublic Map<String, Object> toBrecObject() {")?;
    writer.ln("\t\tHashMap<String, Object> body = new HashMap<>(1);")?;
    writer.ln("\t\tbody.put(variant, value);")?;
    if let Some(variant) = variant_wrapper {
        writer.ln("\t\tHashMap<String, Object> out = new HashMap<>(1);")?;
        writer.ln(format!("\t\tout.put(\"{variant}\", body);"))?;
        writer.ln("\t\treturn out;")?;
    } else {
        writer.ln("\t\treturn body;")?;
    }
    writer.ln("\t}")?;
    writer.ln("")?;
    writer.ln("\t@Override")?;
    writer.ln("\tpublic boolean equals(Object other) {")?;
    writer.ln(format!("\t\tif (!(other instanceof {class_name})) {{"))?;
    writer.ln("\t\t\treturn false;")?;
    writer.ln("\t\t}")?;
    writer.ln(format!("\t\t{class_name} that = ({class_name}) other;"))?;
    writer.ln(
        "\t\treturn Objects.equals(variant, that.variant) && Objects.deepEquals(value, that.value);",
    )?;
    writer.ln("\t}")?;
    writer.ln("")?;
    writer.ln("\t@Override")?;
    writer.ln("\tpublic int hashCode() {")?;
    writer.ln("\t\tint valueHash = value instanceof byte[] ? Arrays.hashCode((byte[]) value) : Objects.hashCode(value);")?;
    writer.ln("\t\treturn 31 * Objects.hashCode(variant) + valueHash;")?;
    writer.ln("\t}")?;
    writer.ln("}")?;
    Ok(())
}

fn payload_imports(fields: &[JavaField]) -> Vec<&'static str> {
    let mut imports = vec!["java.util.HashMap", "java.util.Map", "java.util.Objects"];
    for field in fields {
        field.collect_imports(&mut imports);
        if field.needs_array_helpers() {
            imports.push("java.util.Arrays");
        }
    }
    imports
}

fn enum_imports(variants: &[SchemePayloadVariant]) -> Result<Vec<&'static str>, Error> {
    let mut imports = vec![
        "java.util.Arrays",
        "java.util.HashMap",
        "java.util.Map",
        "java.util.Objects",
    ];
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
