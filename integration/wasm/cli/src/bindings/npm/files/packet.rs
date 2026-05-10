use crate::*;

/// Generated `packet.ts` file.
///
/// The packet shape is intentionally small: ordered blocks plus an optional
/// payload, matching the Rust `Packet::new(blocks, payload)` constructor used
/// by the protocol crate.
pub struct PacketFile<'a> {
    model: &'a Model,
    imports: Vec<Box<dyn Importable + 'a>>,
}

impl<'a> PacketFile<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self {
            model,
            imports: vec![
                Box::new(BlocksFile::from(model)),
                Box::new(PayloadFile::from(model)),
            ],
        }
    }
}

impl<'a> FileName for PacketFile<'a> {
    const FILE_NAME: &'static str = "packet.ts";
}

impl<'a> ModuleName for PacketFile<'a> {
    const MODULE_NAME: &'static str = "Packet";
}

impl<'a> SourceWritable for PacketFile<'a> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        FileHeader::new(Self::FILE_NAME, self.model).write(writer)?;
        for import in &self.imports {
            writer.ln(import.import_statement())?;
        }
        writer.ln("")?;
        writer.ln("export interface Packet {")?;
        writer.tab();
        writer.ln("blocks: Block[];")?;
        writer.ln("payload?: Payload;")?;
        writer.back();
        writer.ln("}")
    }
}
