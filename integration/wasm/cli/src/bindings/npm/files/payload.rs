use crate::*;

/// Generated `payloads.ts` file.
///
/// It emits included helper types first, then protocol payload declarations,
/// and finally the public `Payload` union accepted by `encodePayload`.
pub struct PayloadFile<'a> {
    model: &'a Model,
}

impl<'a> From<&'a Model> for PayloadFile<'a> {
    fn from(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> FileName for PayloadFile<'a> {
    const FILE_NAME: &'static str = "payloads.ts";
}

impl<'a> ModuleName for PayloadFile<'a> {
    const MODULE_NAME: &'static str = "Payload";
}

impl<'a> SourceWritable for PayloadFile<'a> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(Self::FILE_NAME, self.model).write(writer)?;
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
