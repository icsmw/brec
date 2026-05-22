use crate::*;

/// Marker for the generated npm `index.ts` barrel and native-binding wrapper.
///
/// It imports generated type modules, re-exports them, loads
/// `native/bindings.node`, and exposes the typed encode/decode functions.
pub struct NpmIndexFile;

impl FileName for NpmIndexFile {
    const FILE_NAME: &'static str = "index.ts";
}

impl<'a> SourceWritable for ApiFile<'a, NpmIndexFile> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts(writer)?;
        Ok(())
    }
}

impl<'a> TsWritable for ApiFile<'a, NpmIndexFile> {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(self.file_name(), self.model).write(writer)?;
        writer.ln("declare const require: any;")?;
        for import in self.modules() {
            writer.ln(import.import_statement())?;
        }
        writer.ln("")?;
        for import in self.modules() {
            writer.ln(import.export_statement())?;
        }
        writer.block(
            r#"
const native = require('./native/bindings.node');

function pick(camel: string, snake: string): any {
	const value = native[camel] || native[snake];
	if (typeof value !== 'function') {
		throw new Error(`bindings.node does not export ${camel}/${snake}`);
	}
	return value;
}
"#,
        )?;
        for api in self.apis() {
            api.write_ts(writer)?;
        }
        Ok(())
    }
}
