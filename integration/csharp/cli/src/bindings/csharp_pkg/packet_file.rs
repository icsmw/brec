use super::generated_file::GeneratedFile;
use crate::*;

pub struct PacketFile<'a> {
    model: &'a Model,
}

impl<'a> PacketFile<'a> {
    pub const FILE_NAME: &'static str = "Packet.cs";

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for PacketFile<'_> {
    const FILE_NAME: &'static str = Self::FILE_NAME;
}

impl SourceWritable for PacketFile<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        GeneratedFile {
            model: self.model,
            file_name: Self::FILE_NAME,
        }
        .write_header(writer)?;
        GeneratedFile {
            model: self.model,
            file_name: Self::FILE_NAME,
        }
        .write_namespace(writer)?;
        writer.ln("")?;
        write_packet_class(writer)?;
        writer.ln("")?;
        write_bindings(writer)
    }
}

fn write_packet_class(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.ln("public sealed class Packet")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("public IReadOnlyList<Block> Blocks { get; }")?;
    writer.ln("public Payload? Payload { get; }")?;
    writer.ln("")?;
    writer.ln("public Packet(IReadOnlyList<Block> blocks, Payload? payload)")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("Blocks = blocks;")?;
    writer.ln("Payload = payload;")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("internal static Packet FromNative(ValueHandle handle)")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("using var blocksValue = NativeValue.GetField(handle, \"blocks\");")?;
    writer.ln(
        "var blocks = NativeValue.AsList(blocksValue, static item => Block.FromNative(item));",
    )?;
    writer.ln("using var payloadValue = NativeValue.GetField(handle, \"payload\");")?;
    writer.ln("var payload = NativeValue.Kind(payloadValue) == NativeValueKind.Null ? null : Payload.FromNative(payloadValue);")?;
    writer.ln("return new Packet(blocks, payload);")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("internal ValueHandle ToNative()")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("var obj = NativeValue.NewObject();")?;
    writer.ln("try")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln(
        "using (var blocks = NativeValue.FromList(Blocks, static block => block.ToNative()))",
    )?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("NativeValue.PutField(obj, \"blocks\", blocks);")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("using (var payload = Payload is null ? NativeValue.Null() : Payload.ToNative())")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("NativeValue.PutField(obj, \"payload\", payload);")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("return obj;")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("catch")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("obj.Dispose();")?;
    writer.ln("throw;")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")
}

fn write_bindings(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.ln("public static class PacketBindings")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("public static Packet DecodePacket(byte[] bytes)")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("using var native = NativeValue.FromRaw(NativeBindings.decode_packet(bytes, (UIntPtr)bytes.Length), \"decode packet failed\");")?;
    writer.ln("return Packet.FromNative(native);")?;
    writer.back();
    writer.ln("}")?;
    writer.ln("")?;
    writer.ln("public static byte[] EncodePacket(Packet packet)")?;
    writer.ln("{")?;
    writer.tab();
    writer.ln("using var native = packet.ToNative();")?;
    writer.ln(
        "var ptr = NativeBindings.encode_packet(native.DangerousGetHandle(), out var outLen);",
    )?;
    writer.ln("return BindingBytes.TakeBytes(ptr, outLen, \"encode packet failed\");")?;
    writer.back();
    writer.ln("}")?;
    writer.back();
    writer.ln("}")
}
