use crate::*;

/// Marker for the generated Rust `src/lib.rs` binding file.
///
/// The actual content is rendered through `ApiFile<BindingsLibFile>` so the
/// same API list can be shared with the Java entry point.
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
        writer.ln("use jni::{")?;
        writer.tab();
        writer.ln("JNIEnv,")?;
        writer.ln("objects::{JByteArray, JClass, JObject},")?;
        writer.ln("sys::{jbyteArray, jobject},")?;
        writer.back();
        writer.ln("};")?;
        writer.ln("use protocol::{Block, Packet, Payload};")?;
        writer.ln("")?;
        writer.ln("fn throw_runtime(env: &mut JNIEnv<'_>, message: impl AsRef<str>) {")?;
        writer.tab();
        writer.ln(r#"let _ = env.throw_new("java/lang/RuntimeException", message.as_ref());"#)?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
