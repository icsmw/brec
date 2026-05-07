use crate::*;
use std::fmt;

pub struct PayloadFile<'a> {
    model: &'a Model,
}

impl<'a> From<&'a Model> for PayloadFile<'a> {
    fn from(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> Module for PayloadFile<'a> {
    const FILE_NAME: &'static str = "payloads.ts";
    const MODULE_NAME: &'static str = "Payload";
}

impl<'a> FormatterWritable for PayloadFile<'a> {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result {
        FileHeader::new(Self::FILE_NAME, &self.model.package).write(writer)?;
        for included in &self.model.included_types {
            included.write(writer)?;
        }
        for payload in &self.model.payloads {
            payload.write(writer)?;
        }
        self.model.payload_union.write(writer)?;
        Ok(())
    }
}
