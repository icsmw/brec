use super::file::{JavaFile, JavaPackage};
use crate::*;

pub(super) struct ClientFile<'a> {
    model: &'a Model,
}

impl<'a> ClientFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Root, "Client.java", |writer| {
            writer.ln("import com.icsmw.brec.block.Block;")?;
            writer.ln("import com.icsmw.brec.payload.Payload;")?;
            writer.ln("")?;
            writer.ln("public final class Client {")?;
            writer.tab();
            writer.ln("static {")?;
            writer.tab();
            writer.ln(r#"System.loadLibrary("bindings");"#)?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("private Client() {}")?;
            writer.ln("")?;
            writer.ln("private static native Object decodeBlockNative(byte[] bytes);")?;
            writer.ln("private static native byte[] encodeBlockNative(Object block);")?;
            writer.ln("private static native Object decodePayloadNative(byte[] bytes);")?;
            writer.ln("private static native byte[] encodePayloadNative(Object payload);")?;
            writer.ln("private static native Object decodePacketNative(byte[] bytes);")?;
            writer.ln("private static native byte[] encodePacketNative(Object packet);")?;
            writer.ln("")?;
            writer.ln("public static Block decodeBlock(byte[] bytes) {")?;
            writer.tab();
            writer.ln("return Block.fromBrecObject(decodeBlockNative(bytes));")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static byte[] encodeBlock(Block block) {")?;
            writer.tab();
            writer.ln("return encodeBlockNative(block.toBrecObject());")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static Payload decodePayload(byte[] bytes) {")?;
            writer.tab();
            writer.ln("return Payload.fromBrecObject(decodePayloadNative(bytes));")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static byte[] encodePayload(Payload payload) {")?;
            writer.tab();
            writer.ln("return encodePayloadNative(payload.toBrecObject());")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static Packet decodePacket(byte[] bytes) {")?;
            writer.tab();
            writer.ln("return Packet.fromBrecObject(decodePacketNative(bytes));")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("")?;
            writer.ln("public static byte[] encodePacket(Packet packet) {")?;
            writer.tab();
            writer.ln("return encodePacketNative(packet.toBrecObject());")?;
            writer.back();
            writer.ln("}")?;
            writer.back();
            writer.ln("}")
        })
    }
}
