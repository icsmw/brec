use crate::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
    name: String,
    ty: Type,
}

impl TypeAlias {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

impl FormatterWritable for TypeAlias {
    fn write(&self, writer: &mut crate::FormatterWriter) -> fmt::Result {
        match &self.ty {
            Type::Union(items) if !items.is_empty() => {
                writer.write(format!("export type {} = ", self.name))?;
                for (idx, item) in items.iter().enumerate() {
                    let sep = if idx + 1 == items.len() { ";" } else { " | " };
                    item.write(writer)?;
                    writer.write(sep)?;
                }
                writer.ln("")
            }
            _ => {
                writer.write(format!("export type {} = ", self.name))?;
                self.ty.write(writer)?;
                writer.ln(";")
            }
        }
    }
}
