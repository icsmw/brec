use crate::*;
use std::fmt;

pub struct PacketFile<'a> {
    model: &'a Model,
    imports: Vec<&'a dyn Importable>,
}

impl<'a> PacketFile<'a> {
    pub const FILE_NAME: &'static str = "packet.ts";
    pub fn new(model: &'a Model, imports: Vec<&'a dyn Importable>) -> Self {
        Self { model, imports }
    }
}

impl<'a> FormatterWritable for PacketFile<'a> {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result {
        FileHeader::new(Self::FILE_NAME, &self.model.package).write(writer)?;
        for import in &self.imports {
            writer.ln(import.import_statement())?;
        }
        writer.ln("")?;
        writer.ln("export interface Packet {{")?;
        writer.tab();
        writer.ln("blocks: Block[];")?;
        writer.ln("payload?: Payload;")?;
        writer.back();
        writer.ln("}}")
    }
}
