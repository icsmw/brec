use crate::{Error, SourceWritable};

use super::field::Field;

/// Generated TypeScript interface declaration.
///
/// Interfaces are used only for named Rust struct shapes, which gives
/// downstream TypeScript consumers readable property completions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    name: String,
    fields: Vec<Field>,
}

impl Interface {
    pub fn new(name: impl Into<String>, fields: Vec<Field>) -> Self {
        Self {
            name: name.into(),
            fields,
        }
    }
}

impl SourceWritable for Interface {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        writer.ln(format!("export interface {} {{", self.name))?;
        writer.tab();
        for field in &self.fields {
            field.write(writer)?;
        }
        writer.back();
        writer.ln("}")
    }
}
