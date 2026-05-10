use crate::*;

/// Marker for the generated npm `index.ts` barrel and wasm-pack wrapper.
///
/// It imports generated type modules, re-exports them, loads target-specific
/// `wasmjs.js`, and exposes typed encode/decode functions.
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
        match self.target {
            WasmTarget::Node => writer.ln("declare const require: any;")?,
            WasmTarget::Browser => {
                writer.ln("import initWasm, * as wasm from \"./wasmjs.js\";")?;
                writer.ln("export { initWasm };")?;
            }
        }
        for import in self.modules() {
            writer.ln(import.import_statement())?;
        }
        writer.ln("")?;
        for import in self.modules() {
            writer.ln(import.export_statement())?;
        }
        if self.target == WasmTarget::Node {
            writer.ln("const wasm = require('./wasmjs.js');")?;
        }
        writer.ln("const wasmApi = wasm as Record<string, unknown>;")?;
        writer.ln("")?;
        writer.ln("function pick(camel: string, snake: string): any {")?;
        writer.tab();
        writer.ln("const value = wasmApi[camel] || wasmApi[snake];")?;
        writer.ln("if (typeof value !== 'function') {")?;
        writer.tab();
        writer.ln("throw new Error(`wasm package does not export ${camel}/${snake}`);")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("return value;")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in self.apis() {
            api.write_ts_browser(writer)?
        }
        Ok(())
    }
}
