use crate::*;

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

impl SourceWritable for TypeAlias {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        match &self.ty {
            Type::Union(items) if !items.is_empty() => {
                writer.write(format!("export type {} = ", self.name))?;
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" | ")?;
                    }
                    item.write(writer)?;
                }
                writer.ln(";")
            }
            _ => {
                writer.write(format!("export type {} = ", self.name))?;
                self.ty.write(writer)?;
                writer.ln(";")
            }
        }
    }
}
