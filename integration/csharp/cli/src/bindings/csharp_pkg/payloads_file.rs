use super::generated_file::GeneratedFile;
use crate::*;

pub struct PayloadsFile<'a> {
    model: &'a Model,
}

impl<'a> PayloadsFile<'a> {
    pub const FILE_NAME: &'static str = "Payloads.cs";

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for PayloadsFile<'_> {
    const FILE_NAME: &'static str = Self::FILE_NAME;
}

impl SourceWritable for PayloadsFile<'_> {
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

        for included in &self.model.included_types {
            included.write(writer)?;
            writer.ln("")?;
        }

        self.write_payload_base(writer)?;
        writer.ln("")?;

        for payload in &self.model.payloads {
            payload.write_with_parent(writer, "Payload")?;
            writer.ln("")?;
        }

        if self.model.default_payloads {
            self.write_default_payload(writer, "BytesPayload", "Bytes", "byte[]", "Value")?;
            writer.ln("")?;
            self.write_default_payload(writer, "StringPayload", "String", "string", "Value")?;
            writer.ln("")?;
        }

        self.write_bindings(writer)
    }
}

impl PayloadsFile<'_> {
    fn write_payload_base(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("public abstract class Payload")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("private protected Payload() { }")?;
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
        for payload in &self.model.payloads {
            writer.ln(format!("case {} value:", payload.name))?;
            writer.tab();
            writer.ln("using (var native = value.ToNativeObject())")?;
            writer.ln("{")?;
            writer.tab();
            writer.ln(format!(
                "NativeValue.PutField(obj, \"{}\", native);",
                payload.key
            ))?;
            writer.back();
            writer.ln("}")?;
            writer.ln("return obj;")?;
            writer.back();
        }
        if self.model.default_payloads {
            for (name, key) in [("BytesPayload", "Bytes"), ("StringPayload", "String")] {
                writer.ln(format!("case {name} value:"))?;
                writer.tab();
                writer.ln("using (var native = value.ToNativeObject())")?;
                writer.ln("{")?;
                writer.tab();
                writer.ln(format!("NativeValue.PutField(obj, \"{key}\", native);"))?;
                writer.back();
                writer.ln("}")?;
                writer.ln("return obj;")?;
                writer.back();
            }
        }
        writer.ln("default:")?;
        writer.tab();
        writer.ln(
            "throw new InvalidOperationException($\"Unsupported payload type {GetType().Name}\");",
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
        writer.ln("internal static Payload FromNative(ValueHandle handle)")?;
        writer.ln("{")?;
        writer.tab();
        for payload in &self.model.payloads {
            writer.ln(format!(
                "if (NativeValue.HasField(handle, \"{}\"))",
                payload.key
            ))?;
            writer.ln("{")?;
            writer.tab();
            writer.ln(format!(
                "using var inner = NativeValue.GetField(handle, \"{}\");",
                payload.key
            ))?;
            writer.ln(format!("return {}.FromNativeObject(inner);", payload.name))?;
            writer.back();
            writer.ln("}")?;
        }
        if self.model.default_payloads {
            writer.ln("if (NativeValue.HasField(handle, \"Bytes\"))")?;
            writer.ln("{")?;
            writer.tab();
            writer.ln("using var inner = NativeValue.GetField(handle, \"Bytes\");")?;
            writer.ln("return BytesPayload.FromNativeObject(inner);")?;
            writer.back();
            writer.ln("}")?;
            writer.ln("if (NativeValue.HasField(handle, \"String\"))")?;
            writer.ln("{")?;
            writer.tab();
            writer.ln("using var inner = NativeValue.GetField(handle, \"String\");")?;
            writer.ln("return StringPayload.FromNativeObject(inner);")?;
            writer.back();
            writer.ln("}")?;
        }
        writer.ln("throw new InvalidOperationException(\"Unknown payload variant\");")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")
    }

    fn write_default_payload(
        &self,
        writer: &mut SourceWriter,
        name: &str,
        _key: &str,
        ty: &str,
        field: &str,
    ) -> Result<(), Error> {
        writer.ln(format!("public sealed class {name} : Payload"))?;
        writer.ln("{")?;
        writer.tab();
        writer.ln(format!("public {ty} {field} {{ get; }}"))?;
        writer.ln("")?;
        writer.ln(format!("public {name}({ty} value)"))?;
        writer.ln("{")?;
        writer.tab();
        writer.ln(format!("{field} = value;"))?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln(format!(
            "internal static {name} FromNativeObject(ValueHandle handle)"
        ))?;
        writer.ln("{")?;
        writer.tab();
        let read = if ty == "byte[]" {
            "NativeValue.AsList(handle, static item => NativeValue.AsByte(item)).ToArray()"
        } else {
            "NativeValue.AsString(handle)"
        };
        writer.ln(format!("return new {name}({read});"))?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("internal override ValueHandle ToNativeObject()")?;
        writer.ln("{")?;
        writer.tab();
        let write = if ty == "byte[]" {
            "NativeValue.FromList(Value, static item => NativeValue.FromByte(item))"
        } else {
            "NativeValue.FromString(Value)"
        };
        writer.ln(format!("return {write};"))?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")
    }

    fn write_bindings(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("public static class PayloadBindings")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("public static Payload DecodePayload(byte[] bytes)")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("using var native = NativeValue.FromRaw(NativeBindings.decode_payload(bytes, (UIntPtr)bytes.Length), \"decode payload failed\");")?;
        writer.ln("return Payload.FromNative(native);")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("public static byte[] EncodePayload(Payload payload)")?;
        writer.ln("{")?;
        writer.tab();
        writer.ln("using var native = payload.ToNative();")?;
        writer.ln(
            "var ptr = NativeBindings.encode_payload(native.DangerousGetHandle(), out var outLen);",
        )?;
        writer.ln("return BindingBytes.TakeBytes(ptr, outLen, \"encode payload failed\");")?;
        writer.back();
        writer.ln("}")?;
        writer.back();
        writer.ln("}")
    }
}
