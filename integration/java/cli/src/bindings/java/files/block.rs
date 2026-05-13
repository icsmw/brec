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
            writer.ln("public interface Block {")?;
            writer.tab();
            writer.ln("Map<String, Object> toBrecObject();")?;
            writer.ln("")?;
            writer.ln("static Block fromBrecObject(Object value) {")?;
            writer.tab();
            writer.ln("return BlockSupport.fromBrecObject(value);")?;
            writer.back();
            writer.ln("}")?;
            writer.back();
            writer.ln("}")
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
                writer.ln("final class BlockSupport {")?;
                writer.tab();
                writer.ln("private BlockSupport() {}")?;
                writer.ln("")?;
                writer.ln("static Block fromBrecObject(Object value) {")?;
                writer.tab();
                writer.ln("Map<?, ?> map = (Map<?, ?>) value;")?;
                writer.ln("Map.Entry<?, ?> entry = map.entrySet().iterator().next();")?;
                writer.ln("String variant = (String) entry.getKey();")?;
                writer.ln("Object inner = entry.getValue();")?;
                writer.ln("switch (variant) {")?;
                writer.tab();
                for block in &self.scheme.blocks {
                    writer.ln(format!(
                        r#"case "{}": return {}.fromBrecObject(inner);"#,
                        block.fullname, block.fullname
                    ))?;
                }
                writer.ln(
                    r#"default: throw new IllegalArgumentException("unknown block: " + variant);"#,
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
    writer.tab();
    for field in fields {
        writer.ln(format!("public {} {};", field.ty, field.name))?;
    }
    writer.ln("")?;
    writer.ln(format!("public {class_name}() {{}}"))?;
    writer.ln("")?;
    writer.write(format!("private {class_name}("))?;
    write_args(writer, fields)?;
    writer.ln(") {")?;
    writer.tab();
    for field in fields {
        writer.ln(format!("this.{0} = {0};", field.name))?;
    }
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    if fields
        .iter()
        .any(JavaField::needs_unchecked_cast_suppression)
    {
        writer.ln(r#"@SuppressWarnings("unchecked")"#)?;
    }
    writer.ln(format!(
        "static {class_name} fromBrecObject(Object value) {{"
    ))?;
    writer.tab();
    writer.ln("Map<?, ?> map = (Map<?, ?>) value;")?;
    writer.write(format!("return new {class_name}("))?;
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            writer.write(", ")?;
        }
        writer.write(field.from_brec_expr(&format!(r#"map.get("{}")"#, field.name)))?;
    }
    writer.ln(");")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("public Map<String, Object> toBrecObject() {")?;
    writer.tab();
    writer.ln(format!(
        "HashMap<String, Object> body = new HashMap<>({});",
        fields.len()
    ))?;
    for field in fields {
        writer.ln(format!(
            r#"body.put("{}", {});"#,
            field.name,
            field.to_brec_expr()
        ))?;
    }
    if let Some(variant) = variant_wrapper {
        writer.ln("HashMap<String, Object> out = new HashMap<>(1);")?;
        writer.ln(format!(r#"out.put("{variant}", body);"#))?;
        writer.ln("return out;")?;
    } else {
        writer.ln("return body;")?;
    }
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    write_equals_hash_code(writer, class_name, fields)?;
    writer.back();
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
    writer.ln("@Override")?;
    writer.ln("public boolean equals(Object other) {")?;
    writer.tab();
    writer.ln(format!("if (!(other instanceof {class_name})) {{"))?;
    writer.tab();
    writer.ln("return false;")?;
    writer.back();
    writer.ln("}")?;
    writer.ln(format!("{class_name} that = ({class_name}) other;"))?;
    if fields.is_empty() {
        writer.ln("return true;")?;
    } else {
        writer.write("return ")?;
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                writer.write(" && ")?;
            }
            writer.write(field.equals_expr("that"))?;
        }
        writer.ln(";")?;
    }
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("@Override")?;
    writer.ln("public int hashCode() {")?;
    writer.tab();
    writer.ln("int result = 1;")?;
    for field in fields {
        writer.ln(format!("result = 31 * result + {};", field.hash_expr()))?;
    }
    writer.ln("return result;")?;
    writer.back();
    writer.ln("}")?;
    Ok(())
}
