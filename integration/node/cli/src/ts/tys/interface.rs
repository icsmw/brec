use crate::FormatterWritable;

use super::field::Field;
use std::fmt;

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

impl FormatterWritable for Interface {
    fn write(&self, writer: &mut crate::FormatterWriter) -> fmt::Result {
        writer.ln(format!("export interface {} {{", self.name))?;
        writer.tab();
        for field in &self.fields {
            field.write(writer)?;
        }
        writer.back();
        writer.ln("}")
    }
}
