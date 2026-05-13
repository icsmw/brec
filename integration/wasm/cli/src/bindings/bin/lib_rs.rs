use crate::*;

/// Marker for the generated Rust `src/lib.rs` WASM binding file.
///
/// The actual content is rendered through `ApiFile<BindingsLibFile>` so the
/// same API list can be shared with the TypeScript entry point.
pub struct BindingsLibFile;

impl FileName for BindingsLibFile {
    const FILE_NAME: &'static str = "lib.rs";
}

impl<'a> SourceWritable for ApiFile<'a, BindingsLibFile> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_rust(writer)?;
        Ok(())
    }
}

impl<'a> RustWritable for ApiFile<'a, BindingsLibFile> {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(self.file_name(), self.model).write(writer)?;
        writer.ln("use protocol::{Block, Packet, Payload};")?;
        writer.ln("use wasm_bindgen::prelude::*;")?;
        writer.ln("")?;
        writer.ln("fn to_js_error(err: impl std::fmt::Display) -> JsValue {")?;
        writer.tab();
        writer.ln("JsValue::from_str(&err.to_string())")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
