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
        writer.block(
            r#"
use jni::{
	JNIEnv,
	objects::{JByteArray, JClass, JObject},
	sys::{jbyteArray, jobject},
};
use protocol::{Block, Packet, Payload};

fn throw_runtime(env: &mut JNIEnv<'_>, message: impl AsRef<str>) {
	let _ = env.throw_new("java/lang/RuntimeException", message.as_ref());
}
"#,
        )?;
        for api in self.apis() {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
