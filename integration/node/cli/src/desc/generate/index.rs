use crate::*;
use std::fmt;

pub struct IndexFile<'a> {
    exports: Vec<&'a dyn Exportable>,
    model: &'a Model,
}

impl<'a> IndexFile<'a> {
    pub const FILE_NAME: &'static str = "index.ts";
    pub fn new(model: &'a Model, exports: Vec<&'a dyn Exportable>) -> Self {
        Self { exports, model }
    }
}

impl<'a> FormatterWritable for IndexFile<'a> {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result {
        FileHeader::new(Self::FILE_NAME, &self.model.package).write(writer)?;
        for export in &self.exports {
            writer.ln(export.export_statement())?;
        }
        Ok(())
    }
}
