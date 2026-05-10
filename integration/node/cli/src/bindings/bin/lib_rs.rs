use crate::*;

/// Marker for the generated Rust `src/lib.rs` binding file.
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
        writer.ln("use napi::bindgen_prelude::{Buffer, Error, Result, Status};")?;
        writer.ln("use napi::{Env, Unknown};")?;
        writer.ln("use napi_derive::napi;")?;
        writer.ln("use protocol::{Block, Packet, Payload};")?;
        writer.ln("")?;
        writer
            .ln("fn to_napi_error(prefix: &'static str, err: impl std::fmt::Display) -> Error {")?;
        writer.tab();
        writer.ln(r#"Error::new(Status::GenericFailure, format!("{prefix}: {err}"))"#)?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
