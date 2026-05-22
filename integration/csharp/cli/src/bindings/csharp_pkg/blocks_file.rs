use super::generated_file::GeneratedFile;
use crate::*;

pub struct BlocksFile<'a> {
    model: &'a Model,
}

impl<'a> BlocksFile<'a> {
    pub const FILE_NAME: &'static str = "Blocks.cs";

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for BlocksFile<'_> {
    const FILE_NAME: &'static str = Self::FILE_NAME;
}

impl SourceWritable for BlocksFile<'_> {
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
        self.write_block_base(writer)?;
        writer.ln("")?;
        for block in &self.model.blocks {
            block.write(writer)?;
            writer.ln("")?;
        }
        self.write_bindings(writer)
    }
}

impl BlocksFile<'_> {
    fn write_block_base(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(
            r#"
public abstract class Block
{
	private protected Block() { }
	internal abstract ValueHandle ToNativeObject();

	internal ValueHandle ToNative()
	{
		var obj = NativeValue.NewObject();
		try
		{
			switch (this)
			{
"#,
        )?;
        for block in &self.model.blocks {
            writer.block(format!(
                r#"
			case {} value:
				using (var native = value.ToNativeObject())
				{{
					NativeValue.PutField(obj, "{}", native);
				}}
				return obj;
"#,
                block.name, block.key
            ))?;
        }
        writer.block(
            r#"
			default:
				throw new InvalidOperationException($"Unsupported block type {GetType().Name}");
			}
		}
		catch
		{
			obj.Dispose();
			throw;
		}
	}

	internal static Block FromNative(ValueHandle handle)
	{
"#,
        )?;
        for block in &self.model.blocks {
            writer.block(format!(
                r#"
		if (NativeValue.HasField(handle, "{}"))
		{{
			using var inner = NativeValue.GetField(handle, "{}");
			return {}.FromNativeObject(inner);
		}}
"#,
                block.key, block.key, block.name
            ))?;
        }
        writer.block(
            r#"
		throw new InvalidOperationException("Unknown block variant");
	}
}
"#,
        )
    }

    fn write_bindings(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(
            r#"
public static class BlockBindings
{
	public static Block DecodeBlock(byte[] bytes)
	{
		using var native = NativeValue.FromRaw(NativeBindings.decode_block(bytes, (UIntPtr)bytes.Length), "decode block failed");
		return Block.FromNative(native);
	}

	public static byte[] EncodeBlock(Block block)
	{
		using var native = block.ToNative();
		var ptr = NativeBindings.encode_block(native.DangerousGetHandle(), out var outLen);
		return BindingBytes.TakeBytes(ptr, outLen, "encode block failed");
	}
}
"#,
        )
    }
}
