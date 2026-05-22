use super::file::{JavaFile, JavaPackage, write_imports};
use crate::*;
use brec_scheme::{SchemeBlock, SchemeFile};

pub(super) struct BlockInterfaceFile<'a> {
    model: &'a Model,
}

impl<'a> BlockInterfaceFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Block, "Block.java", |writer| {
            write_imports(writer, &["java.util.Map"])?;
            writer.block(
                r#"
public interface Block {
	Map<String, Object> toBrecObject();

	static Block fromBrecObject(Object value) {
		return BlockSupport.fromBrecObject(value);
	}
}
"#,
            )
        })
    }
}

pub(super) struct BlockSupportFile<'a> {
    model: &'a Model,
    scheme: &'a SchemeFile,
}

impl<'a> BlockSupportFile<'a> {
    pub(super) fn new(model: &'a Model, scheme: &'a SchemeFile) -> Self {
        Self { model, scheme }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(
            self.model,
            JavaPackage::Block,
            "BlockSupport.java",
            |writer| {
                write_imports(writer, &["java.util.Map"])?;
                writer.block(
                    r#"
final class BlockSupport {
	private BlockSupport() {}

	static Block fromBrecObject(Object value) {
		Map<?, ?> map = (Map<?, ?>) value;
		Map.Entry<?, ?> entry = map.entrySet().iterator().next();
		String variant = (String) entry.getKey();
		Object inner = entry.getValue();
		switch (variant) {
"#,
                )?;
                for block in &self.scheme.blocks {
                    writer.ln(format!(
                        "\t\t\tcase \"{}\": return {}.fromBrecObject(inner);",
                        block.fullname, block.fullname
                    ))?;
                }
                writer.ln(
                    "\t\t\tdefault: throw new IllegalArgumentException(\"unknown block: \" + variant);",
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

pub(super) struct BlockFile<'a> {
    model: &'a Model,
    block: &'a SchemeBlock,
}

impl<'a> BlockFile<'a> {
    pub(super) fn new(model: &'a Model, block: &'a SchemeBlock) -> Self {
        Self { model, block }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        let fields = self
            .block
            .fields
            .iter()
            .map(JavaField::from_block)
            .collect::<Result<Vec<_>, _>>()?;
        let class_name = self.block.fullname.clone();
        JavaFile::new(
            self.model,
            JavaPackage::Block,
            format!("{class_name}.java"),
            |writer| {
                write_imports(writer, &block_imports(&fields))?;
                writer.ln(format!(
                    "public final class {class_name} implements Block {{"
                ))?;
                write_fields_class_body(writer, &class_name, &fields, Some(&class_name))
            },
        )
    }
}

pub(super) fn write_fields_class_body(
    writer: &mut SourceWriter,
    class_name: &str,
    fields: &[JavaField],
    variant_wrapper: Option<&str>,
) -> Result<(), Error> {
    for field in fields {
        writer.ln(format!("\tpublic {} {};", field.ty, field.name))?;
    }
    writer.ln("")?;
    writer.ln(format!("\tpublic {class_name}() {{}}"))?;
    writer.ln("")?;
    writer.write(format!("\tprivate {class_name}("))?;
    write_args(writer, fields)?;
    writer.ln(") {")?;
    for field in fields {
        writer.ln(format!("\t\tthis.{0} = {0};", field.name))?;
    }
    writer.ln("\t}")?;
    writer.ln("")?;
    if fields
        .iter()
        .any(JavaField::needs_unchecked_cast_suppression)
    {
        writer.ln("\t@SuppressWarnings(\"unchecked\")")?;
    }
    writer.ln(format!(
        "\tstatic {class_name} fromBrecObject(Object value) {{"
    ))?;
    writer.ln("\t\tMap<?, ?> map = (Map<?, ?>) value;")?;
    writer.write(format!("\t\treturn new {class_name}("))?;
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            writer.write(", ")?;
        }
        writer.write(field.from_brec_expr(&format!(r#"map.get("{}")"#, field.name)))?;
    }
    writer.ln(");")?;
    writer.ln("\t}")?;
    writer.ln("")?;
    writer.ln("\tpublic Map<String, Object> toBrecObject() {")?;
    writer.ln(format!(
        "\t\tHashMap<String, Object> body = new HashMap<>({});",
        fields.len()
    ))?;
    for field in fields {
        writer.ln(format!(
            "\t\tbody.put(\"{}\", {});",
            field.name,
            field.to_brec_expr()
        ))?;
    }
    if let Some(variant) = variant_wrapper {
        writer.ln("\t\tHashMap<String, Object> out = new HashMap<>(1);")?;
        writer.ln(format!("\t\tout.put(\"{variant}\", body);"))?;
        writer.ln("\t\treturn out;")?;
    } else {
        writer.ln("\t\treturn body;")?;
    }
    writer.ln("\t}")?;
    writer.ln("")?;
    write_equals_hash_code(writer, class_name, fields)?;
    writer.ln("}")?;
    Ok(())
}

fn block_imports(fields: &[JavaField]) -> Vec<&'static str> {
    let mut imports = vec!["java.util.HashMap", "java.util.Map", "java.util.Objects"];
    for field in fields {
        field.collect_imports(&mut imports);
        if field.needs_array_helpers() {
            imports.push("java.util.Arrays");
        }
    }
    imports
}

pub(super) fn write_equals_hash_code(
    writer: &mut SourceWriter,
    class_name: &str,
    fields: &[JavaField],
) -> Result<(), Error> {
    writer.ln("\t@Override")?;
    writer.ln("\tpublic boolean equals(Object other) {")?;
    writer.ln(format!("\t\tif (!(other instanceof {class_name})) {{"))?;
    writer.ln("\t\t\treturn false;")?;
    writer.ln("\t\t}")?;
    writer.ln(format!("\t\t{class_name} that = ({class_name}) other;"))?;
    if fields.is_empty() {
        writer.ln("\t\treturn true;")?;
    } else {
        writer.write("\t\treturn ")?;
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                writer.write(" && ")?;
            }
            writer.write(field.equals_expr("that"))?;
        }
        writer.ln(";")?;
    }
    writer.ln("\t}")?;
    writer.ln("")?;
    writer.ln("\t@Override")?;
    writer.ln("\tpublic int hashCode() {")?;
    writer.ln("\t\tint result = 1;")?;
    for field in fields {
        writer.ln(format!("\t\tresult = 31 * result + {};", field.hash_expr()))?;
    }
    writer.ln("\t\treturn result;")?;
    writer.ln("\t}")?;
    Ok(())
}
