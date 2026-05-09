use crate::*;

impl<'a> FormattableTs for ApiModule<'a> {
    fn write_ts(&self, writer: &mut FormatterWriter) -> std::fmt::Result {
        writer.ln("declare const require: any;")?;
        for import in &self.mods {
            writer.ln(import.import_statement())?;
        }
        writer.ln("")?;
        for import in &self.mods {
            writer.ln(import.export_statement())?;
        }
        writer.ln("const native = require('./native/bindings.node');")?;
        writer.ln("")?;
        writer.ln("function pick(camel: string, snake: string): any {")?;
        writer.tab();
        writer.ln("const value = native[camel] || native[snake];")?;
        writer.ln("if (typeof value !== 'function') {")?;
        writer.tab();
        writer.ln("throw new Error(`bindings.node does not export ${camel}/${snake}`);")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("return value;")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in &self.apis {
            api.write_ts(writer)?;
        }
        Ok(())
    }
}
