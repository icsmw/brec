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
        writer.block(
            r#"
use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi::{Env, Unknown};
use napi_derive::napi;
use protocol::{Block, Packet, Payload};

fn to_napi_error(prefix: &'static str, err: impl std::fmt::Display) -> Error {
	Error::new(Status::GenericFailure, format!("{prefix}: {err}"))
}
"#,
        )?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
