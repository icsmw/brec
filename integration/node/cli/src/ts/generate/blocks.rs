use crate::*;
use std::fmt;

pub struct BlocksFile<'a> {
    model: &'a Model,
}

impl<'a> From<&'a Model> for BlocksFile<'a> {
    fn from(model: &'a Model) -> Self {
        BlocksFile { model }
    }
}

impl<'a> Module for BlocksFile<'a> {
    const FILE_NAME: &'static str = "blocks.ts";
    const MODULE_NAME: &'static str = "Block";
}

impl<'a> FormatterWritable for BlocksFile<'a> {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result {
        FileHeader::new(Self::FILE_NAME, &self.model.package).write(writer)?;
        for block in &self.model.blocks {
            block.interface().write(writer)?;
        }
        self.model.block_union.write(writer)?;
        Ok(())
    }
}
