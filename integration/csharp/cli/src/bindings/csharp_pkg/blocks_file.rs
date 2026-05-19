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
        writer.ln("public abstract class Block")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("private protected Block() { }")?;
        writer.ln("internal abstract ValueHandle ToNativeObject();")?;
        writer.ln("")?;
        writer.ln("internal ValueHandle ToNative()")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("var obj = NativeValue.NewObject();")?;
        writer.ln("try")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("switch (this)")?;
        writer.ln("{")?;
        writer.tab();
        for block in &self.model.blocks {
            writer.ln(format!("case {} value:", block.name))?;
            writer.tab();
            writer.ln("using (var native = value.ToNativeObject())")?;
            writer.ln("{")?;
            writer.tab();
            writer.ln(format!(
                "NativeValue.PutField(obj, \"{}\", native);",
                block.key
            ))?;
            writer.back();
            writer.ln("}")?;
            writer.ln("return obj;")?;
            writer.back();
        }
        writer.ln("default:")?;
        writer.tab();
        writer.ln(
            "throw new InvalidOperationException($\"Unsupported block type {GetType().Name}\");",
        )?;
        writer.back();
        writer.back();
        writer.ln("}")?;
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
        writer.ln("")?;
        writer.ln("internal static Block FromNative(ValueHandle handle)")?;
        writer.ln("{")?;
        writer.tab();
        for block in &self.model.blocks {
            writer.ln(format!(
                "if (NativeValue.HasField(handle, \"{}\"))",
                block.key
            ))?;
            writer.ln("{")?;
            writer.tab();
            writer.ln(format!(
                "using var inner = NativeValue.GetField(handle, \"{}\");",
                block.key
            ))?;
            writer.ln(format!("return {}.FromNativeObject(inner);", block.name))?;
            writer.back();
            writer.ln("}")?;
        }
        writer.ln("throw new InvalidOperationException(\"Unknown block variant\");")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")
    }

    fn write_bindings(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("public static class BlockBindings")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("public static Block DecodeBlock(byte[] bytes)")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("using var native = NativeValue.FromRaw(NativeBindings.decode_block(bytes, (UIntPtr)bytes.Length), \"decode block failed\");")?;
        writer.ln("return Block.FromNative(native);")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("public static byte[] EncodeBlock(Block block)")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("using var native = block.ToNative();")?;
        writer.ln(
            "var ptr = NativeBindings.encode_block(native.DangerousGetHandle(), out var outLen);",
        )?;
        writer.ln("return BindingBytes.TakeBytes(ptr, outLen, \"encode block failed\");")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")
    }
}
