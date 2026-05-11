use super::file::{JavaFile, JavaPackage, write_imports};
use crate::*;

pub(super) struct PacketFile<'a> {
    model: &'a Model,
}

impl<'a> PacketFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Root, "Packet.java", |writer| {
            write_imports(
                writer,
                &[
                    "com.icsmw.brec.block.Block",
                    "com.icsmw.brec.payload.Payload",
                    "java.util.ArrayList",
                    "java.util.HashMap",
                    "java.util.List",
                    "java.util.Map",
                ],
            )?;
            writer.ln("public final class Packet {")?;
            writer.tab();
            writer.ln("public List<Block> blocks;")?;
            writer.ln("public Payload payload;")?;
            writer.ln("")?;
            writer.ln("public Packet() {}")?;
            writer.ln("")?;
            writer.ln("public Packet(List<Block> blocks, Payload payload) {")?;
            writer.tab();
            writer.ln("this.blocks = blocks;")?;
            writer.ln("this.payload = payload;")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static Packet withoutPayload(List<Block> blocks) {")?;
            writer.tab();
            writer.ln("return new Packet(blocks, null);")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("static Packet fromBrecObject(Object value) {")?;
            writer.tab();
            writer.ln("Map<?, ?> map = (Map<?, ?>) value;")?;
            writer.ln("List<Block> blocks = new ArrayList<>();")?;
            writer.ln(r#"for (Object block : (List<?>) map.get("blocks")) {"#)?;
            writer.tab();
            writer.ln("blocks.add(Block.fromBrecObject(block));")?;
            writer.back();
            writer.ln("}")?;
            writer.ln(r#"Object payload = map.get("payload");"#)?;
            writer.ln(
                "return new Packet(blocks, payload == null ? null : Payload.fromBrecObject(payload));",
            )?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("Map<String, Object> toBrecObject() {")?;
            writer.tab();
            writer.ln("HashMap<String, Object> out = new HashMap<>();")?;
            writer.ln("ArrayList<Object> encodedBlocks = new ArrayList<>();")?;
            writer.ln("for (Block block : blocks) {")?;
            writer.tab();
            writer.ln("encodedBlocks.add(block.toBrecObject());")?;
            writer.back();
            writer.ln("}")?;
            writer.ln(r#"out.put("blocks", encodedBlocks);"#)?;
            writer.ln(r#"out.put("payload", payload == null ? null : payload.toBrecObject());"#)?;
            writer.ln("return out;")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("@Override")?;
            writer.ln("public boolean equals(Object other) {")?;
            writer.tab();
            writer.ln("return other instanceof Packet && toBrecObject().equals(((Packet) other).toBrecObject());")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("@Override")?;
            writer.ln("public int hashCode() {")?;
            writer.tab();
            writer.ln("return toBrecObject().hashCode();")?;
            writer.back();
            writer.ln("}")?;
            writer.back();
            writer.ln("}")
        })
    }
}
