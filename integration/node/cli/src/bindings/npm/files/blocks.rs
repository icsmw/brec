use crate::*;

pub struct BlocksFile<'a> {
    model: &'a Model,
}

impl<'a> From<&'a Model> for BlocksFile<'a> {
    fn from(model: &'a Model) -> Self {
        BlocksFile { model }
    }
}

impl<'a> FileName for BlocksFile<'a> {
    const FILE_NAME: &'static str = "blocks.ts";
}

impl<'a> ModuleName for BlocksFile<'a> {
    const MODULE_NAME: &'static str = "Block";
}

impl<'a> SourceWritable for BlocksFile<'a> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(Self::FILE_NAME, &self.model).write(writer)?;
        for block in &self.model.blocks {
            block.interface().write(writer)?;
        }
        self.model.block_union.write(writer)?;
        Ok(())
    }
}
