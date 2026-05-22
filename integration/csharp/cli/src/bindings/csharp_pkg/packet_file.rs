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
    writer.block(
        r#"
public sealed class Packet
{
	public IReadOnlyList<Block> Blocks { get; }
	public Payload? Payload { get; }

	public Packet(IReadOnlyList<Block> blocks, Payload? payload)
	{
		Blocks = blocks;
		Payload = payload;
	}

	internal static Packet FromNative(ValueHandle handle)
	{
		using var blocksValue = NativeValue.GetField(handle, "blocks");
		var blocks = NativeValue.AsList(blocksValue, static item => Block.FromNative(item));
		using var payloadValue = NativeValue.GetField(handle, "payload");
		var payload = NativeValue.Kind(payloadValue) == NativeValueKind.Null ? null : Payload.FromNative(payloadValue);
		return new Packet(blocks, payload);
	}

	internal ValueHandle ToNative()
	{
		var obj = NativeValue.NewObject();
		try
		{
			using (var blocks = NativeValue.FromList(Blocks, static block => block.ToNative()))
			{
				NativeValue.PutField(obj, "blocks", blocks);
			}
			using (var payload = Payload is null ? NativeValue.Null() : Payload.ToNative())
			{
				NativeValue.PutField(obj, "payload", payload);
			}
			return obj;
		}
		catch
		{
			obj.Dispose();
			throw;
		}
	}
}
"#,
    )
}

fn write_bindings(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
public static class PacketBindings
{
	public static Packet DecodePacket(byte[] bytes)
	{
		using var native = NativeValue.FromRaw(NativeBindings.decode_packet(bytes, (UIntPtr)bytes.Length), "decode packet failed");
		return Packet.FromNative(native);
	}

	public static byte[] EncodePacket(Packet packet)
	{
		using var native = packet.ToNative();
		var ptr = NativeBindings.encode_packet(native.DangerousGetHandle(), out var outLen);
		return BindingBytes.TakeBytes(ptr, outLen, "encode packet failed");
	}
}
"#,
    )
}
