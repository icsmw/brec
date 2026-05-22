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
        writer.block(
            r#"
public abstract class Payload
{
	private protected Payload() { }
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
        for payload in &self.model.payloads {
            writer.block(format!(
                r#"
			case {} value:
				using (var native = value.ToNativeObject())
				{{
					NativeValue.PutField(obj, "{}", native);
				}}
				return obj;
"#,
                payload.name, payload.key
            ))?;
        }
        if self.model.default_payloads {
            for (name, key) in [("BytesPayload", "Bytes"), ("StringPayload", "String")] {
                writer.block(format!(
                    r#"
			case {name} value:
				using (var native = value.ToNativeObject())
				{{
					NativeValue.PutField(obj, "{key}", native);
				}}
				return obj;
"#
                ))?;
            }
        }
        writer.block(
            r#"
			default:
				throw new InvalidOperationException($"Unsupported payload type {GetType().Name}");
			}
		}
		catch
		{
			obj.Dispose();
			throw;
		}
	}

	internal static Payload FromNative(ValueHandle handle)
	{
"#,
        )?;
        for payload in &self.model.payloads {
            writer.block(format!(
                r#"
		if (NativeValue.HasField(handle, "{}"))
		{{
			using var inner = NativeValue.GetField(handle, "{}");
			return {}.FromNativeObject(inner);
		}}
"#,
                payload.key, payload.key, payload.name
            ))?;
        }
        if self.model.default_payloads {
            writer.block(
                r#"
		if (NativeValue.HasField(handle, "Bytes"))
		{
			using var inner = NativeValue.GetField(handle, "Bytes");
			return BytesPayload.FromNativeObject(inner);
		}
		if (NativeValue.HasField(handle, "String"))
		{
			using var inner = NativeValue.GetField(handle, "String");
			return StringPayload.FromNativeObject(inner);
		}
"#,
            )?;
        }
        writer.block(
            r#"
		throw new InvalidOperationException("Unknown payload variant");
	}
}
"#,
        )
    }

    fn write_default_payload(
        &self,
        writer: &mut SourceWriter,
        name: &str,
        _key: &str,
        ty: &str,
        field: &str,
    ) -> Result<(), Error> {
        let read = if ty == "byte[]" {
            "NativeValue.AsList(handle, static item => NativeValue.AsByte(item)).ToArray()"
        } else {
            "NativeValue.AsString(handle)"
        };
        let write = if ty == "byte[]" {
            "NativeValue.FromList(Value, static item => NativeValue.FromByte(item))"
        } else {
            "NativeValue.FromString(Value)"
        };
        writer.block(format!(
            r#"
public sealed class {name} : Payload
{{
	public {ty} {field} {{ get; }}

	public {name}({ty} value)
	{{
		{field} = value;
	}}

	internal static {name} FromNativeObject(ValueHandle handle)
	{{
		return new {name}({read});
	}}

	internal override ValueHandle ToNativeObject()
	{{
		return {write};
	}}
}}
"#
        ))
    }

    fn write_bindings(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(
            r#"
public static class PayloadBindings
{
	public static Payload DecodePayload(byte[] bytes)
	{
		using var native = NativeValue.FromRaw(NativeBindings.decode_payload(bytes, (UIntPtr)bytes.Length), "decode payload failed");
		return Payload.FromNative(native);
	}

	public static byte[] EncodePayload(Payload payload)
	{
		using var native = payload.ToNative();
		var ptr = NativeBindings.encode_payload(native.DangerousGetHandle(), out var outLen);
		return BindingBytes.TakeBytes(ptr, outLen, "encode payload failed");
	}
}
"#,
        )
    }
}
