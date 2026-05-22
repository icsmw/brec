use super::generated_file::GeneratedFile;
use super::project_file::namespace_name;
use crate::*;

pub struct BindingsFile<'a> {
    model: &'a Model,
}

impl<'a> BindingsFile<'a> {
    pub const FILE_NAME: &'static str = "Bindings.cs";

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for BindingsFile<'_> {
    const FILE_NAME: &'static str = Self::FILE_NAME;
}

impl SourceWritable for BindingsFile<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        GeneratedFile {
            model: self.model,
            file_name: Self::FILE_NAME,
        }
        .write_header(writer)?;
        writer.block(format!(
            r#"
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
using System.Text;

namespace {};
"#,
            namespace_name(&self.model.package)
        ))?;
        write_native_bindings(writer)?;
        writer.ln("")?;
        write_binding_errors(writer)?;
        writer.ln("")?;
        write_value_handle(writer)?;
        writer.ln("")?;
        write_native_value(writer)?;
        writer.ln("")?;
        write_binding_bytes(writer)
    }
}

fn write_native_bindings(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
internal static class NativeBindings
{
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_last_error_message();

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern void bindings_value_free(IntPtr handle);

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern void bindings_bytes_free(IntPtr ptr, UIntPtr len);

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern int bindings_value_kind(IntPtr handle);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_null();
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_bool([MarshalAs(UnmanagedType.I1)] bool value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_u8(byte value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_u16(ushort value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_u32(uint value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_u64(ulong value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_u128(ulong low, ulong high);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_i8(sbyte value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_i16(short value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_i32(int value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_i64(long value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_i128(ulong low, long high);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_f32_bits(uint value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_f64_bits(ulong value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_string([In] byte[] bytes, UIntPtr bytes_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_bytes([In] byte[] bytes, UIntPtr bytes_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_array(UIntPtr capacity);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_object();
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_bool(IntPtr handle, [MarshalAs(UnmanagedType.I1)] out bool value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_u8(IntPtr handle, out byte value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_u16(IntPtr handle, out ushort value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_u32(IntPtr handle, out uint value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_u64(IntPtr handle, out ulong value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_u128(IntPtr handle, out ulong low, out ulong high);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_i8(IntPtr handle, out sbyte value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_i16(IntPtr handle, out short value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_i32(IntPtr handle, out int value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_i64(IntPtr handle, out long value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_i128(IntPtr handle, out ulong low, out long high);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_f32_bits(IntPtr handle, out uint value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_get_f64_bits(IntPtr handle, out ulong value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_get_bytes(IntPtr handle, out UIntPtr out_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern UIntPtr bindings_value_array_len(IntPtr handle);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_array_get(IntPtr handle, UIntPtr index);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_array_push(IntPtr handle, IntPtr value);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_object_has(IntPtr handle, [In] byte[] key, UIntPtr key_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr bindings_value_object_get(IntPtr handle, [In] byte[] key, UIntPtr key_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	[return: MarshalAs(UnmanagedType.I1)]
	internal static extern bool bindings_value_object_put(IntPtr handle, [In] byte[] key, UIntPtr key_len, IntPtr value);

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr decode_block([In] byte[] bytes, UIntPtr bytes_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr encode_block(IntPtr handle, out UIntPtr out_len);

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr decode_payload([In] byte[] bytes, UIntPtr bytes_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr encode_payload(IntPtr handle, out UIntPtr out_len);

	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr decode_packet([In] byte[] bytes, UIntPtr bytes_len);
	[DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
	internal static extern IntPtr encode_packet(IntPtr handle, out UIntPtr out_len);
}
"#,
    )
}

fn write_binding_errors(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
internal static class BindingErrors
{
	internal static string LastErrorOrDefault(string fallback)
	{
		var ptr = NativeBindings.bindings_last_error_message();
		return ptr == IntPtr.Zero ? fallback : Marshal.PtrToStringAnsi(ptr) ?? fallback;
	}
}
"#,
    )
}

fn write_value_handle(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
internal sealed class ValueHandle : SafeHandleZeroOrMinusOneIsInvalid
{
	internal ValueHandle() : base(true) { }
	internal ValueHandle(IntPtr handlePtr) : base(true)
	{
		SetHandle(handlePtr);
	}

	protected override bool ReleaseHandle()
	{
		NativeBindings.bindings_value_free(handle);
		return true;
	}
}
"#,
    )
}

fn write_native_value(writer: &mut SourceWriter) -> Result<(), Error> {
    write_native_value_kind(writer)?;
    writer.ln("")?;
    writer.block("internal static class NativeValue\n{")?;
    write_native_value_constructors(writer)?;
    write_native_value_readers(writer)?;
    write_native_value_objects(writer)?;
    write_native_value_lists(writer)?;
    write_native_value_unsupported(writer)?;
    writer.ln("}")
}

fn write_native_value_kind(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
internal enum NativeValueKind
{
	Null = 0,
	Bool = 1,
	U8 = 2,
	U16 = 3,
	U32 = 4,
	U64 = 5,
	U128 = 6,
	I8 = 7,
	I16 = 8,
	I32 = 9,
	I64 = 10,
	I128 = 11,
	F32Bits = 12,
	F64Bits = 13,
	String = 14,
	Bytes = 15,
	Array = 16,
	Object = 17,
}
"#,
    )
}

fn write_native_value_constructors(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static ValueHandle FromRaw(IntPtr ptr, string fallback)
	{
		if (ptr == IntPtr.Zero)
		{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault(fallback));
		}
		return new ValueHandle(ptr);
	}
"#,
    )?;
    writer.ln("")?;

    writer.block(
        r#"
	internal static NativeValueKind Kind(ValueHandle handle) => (NativeValueKind)NativeBindings.bindings_value_kind(handle.DangerousGetHandle());
	internal static ValueHandle Null() => FromRaw(NativeBindings.bindings_value_null(), "create null failed");
	internal static ValueHandle FromBoolean(bool value) => FromRaw(NativeBindings.bindings_value_bool(value), "create bool failed");
	internal static ValueHandle FromByte(byte value) => FromRaw(NativeBindings.bindings_value_u8(value), "create u8 failed");
	internal static ValueHandle FromUInt16(ushort value) => FromRaw(NativeBindings.bindings_value_u16(value), "create u16 failed");
	internal static ValueHandle FromUInt32(uint value) => FromRaw(NativeBindings.bindings_value_u32(value), "create u32 failed");
	internal static ValueHandle FromUInt64(ulong value) => FromRaw(NativeBindings.bindings_value_u64(value), "create u64 failed");
	internal static ValueHandle FromSByte(sbyte value) => FromRaw(NativeBindings.bindings_value_i8(value), "create i8 failed");
	internal static ValueHandle FromInt16(short value) => FromRaw(NativeBindings.bindings_value_i16(value), "create i16 failed");
	internal static ValueHandle FromInt32(int value) => FromRaw(NativeBindings.bindings_value_i32(value), "create i32 failed");
	internal static ValueHandle FromInt64(long value) => FromRaw(NativeBindings.bindings_value_i64(value), "create i64 failed");
	internal static ValueHandle FromSingle(float value) => FromRaw(NativeBindings.bindings_value_f32_bits(BitConverter.SingleToUInt32Bits(value)), "create f32 failed");
	internal static ValueHandle FromDouble(double value) => FromRaw(NativeBindings.bindings_value_f64_bits(BitConverter.DoubleToUInt64Bits(value)), "create f64 failed");
	internal static ValueHandle FromUInt128(UInt128 value) => FromRaw(NativeBindings.bindings_value_u128((ulong)value, (ulong)(value >> 64)), "create u128 failed");
	internal static ValueHandle FromInt128(Int128 value) => FromRaw(NativeBindings.bindings_value_i128((ulong)value, (long)(value >> 64)), "create i128 failed");

	internal static ValueHandle FromString(string value)
	{
		var bytes = Encoding.UTF8.GetBytes(value);
		return FromRaw(NativeBindings.bindings_value_string(bytes, (UIntPtr)bytes.Length), "create string failed");
	}
	internal static ValueHandle FromBytes(byte[] value) => FromRaw(NativeBindings.bindings_value_bytes(value, (UIntPtr)value.Length), "create bytes failed");
	internal static ValueHandle NewArray(int capacity) => FromRaw(NativeBindings.bindings_value_array((UIntPtr)capacity), "create array failed");
	internal static ValueHandle NewObject() => FromRaw(NativeBindings.bindings_value_object(), "create object failed");
"#,
    )?;
    writer.ln("")
}

fn write_native_value_readers(writer: &mut SourceWriter) -> Result<(), Error> {
    for (name, ty, native) in [
        ("AsBoolean", "bool", "bool"),
        ("AsByte", "byte", "u8"),
        ("AsUInt16", "ushort", "u16"),
        ("AsUInt32", "uint", "u32"),
        ("AsUInt64", "ulong", "u64"),
        ("AsSByte", "sbyte", "i8"),
        ("AsInt16", "short", "i16"),
        ("AsInt32", "int", "i32"),
        ("AsInt64", "long", "i64"),
    ] {
        write_native_value_out_reader(writer, name, ty, native, "read value failed")?;
    }

    write_native_value_u128_reader(writer)?;
    write_native_value_i128_reader(writer)?;
    write_native_value_float_reader(
        writer,
        "AsSingle",
        "uint",
        "f32_bits",
        "read f32 failed",
        "BitConverter.UInt32BitsToSingle(bits)",
    )?;
    write_native_value_float_reader(
        writer,
        "AsDouble",
        "ulong",
        "f64_bits",
        "read f64 failed",
        "BitConverter.UInt64BitsToDouble(bits)",
    )?;
    write_native_value_bytes_reader(writer)?;
    writer.ln(
        "\tinternal static string AsString(ValueHandle handle) => Encoding.UTF8.GetString(AsBytes(handle));",
    )?;
    writer.ln("")
}

fn write_native_value_out_reader(
    writer: &mut SourceWriter,
    name: &str,
    ty: &str,
    native: &str,
    fallback: &str,
) -> Result<(), Error> {
    writer.block(format!(
        r#"
	internal static {ty} {name}(ValueHandle handle)
	{{
		if (!NativeBindings.bindings_value_get_{native}(handle.DangerousGetHandle(), out var value))
		{{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("{fallback}"));
		}}
		return value;
	}}
"#
    ))
}

fn write_native_value_u128_reader(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static UInt128 AsUInt128(ValueHandle handle)
	{
		if (!NativeBindings.bindings_value_get_u128(handle.DangerousGetHandle(), out var low, out var high))
		{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("read u128 failed"));
		}
		return ((UInt128)high << 64) | low;
	}
"#,
    )
}

fn write_native_value_i128_reader(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static Int128 AsInt128(ValueHandle handle)
	{
		if (!NativeBindings.bindings_value_get_i128(handle.DangerousGetHandle(), out var low, out var high))
		{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("read i128 failed"));
		}
		return ((Int128)high << 64) | low;
	}
"#,
    )
}

fn write_native_value_float_reader(
    writer: &mut SourceWriter,
    name: &str,
    bits_ty: &str,
    native: &str,
    fallback: &str,
    convert: &str,
) -> Result<(), Error> {
    let ty = if bits_ty == "uint" { "float" } else { "double" };
    writer.block(format!(
        r#"
	internal static {ty} {name}(ValueHandle handle)
	{{
		if (!NativeBindings.bindings_value_get_{native}(handle.DangerousGetHandle(), out var bits))
		{{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("{fallback}"));
		}}
		return {convert};
	}}
"#
    ))
}

fn write_native_value_bytes_reader(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static byte[] AsBytes(ValueHandle handle)
	{
		var ptr = NativeBindings.bindings_value_get_bytes(handle.DangerousGetHandle(), out var outLen);
		return BindingBytes.TakeBytes(ptr, outLen, "read bytes failed");
	}
"#,
    )
}

fn write_native_value_objects(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static ValueHandle GetField(ValueHandle handle, string key)
	{
		var bytes = Encoding.UTF8.GetBytes(key);
		return FromRaw(NativeBindings.bindings_value_object_get(handle.DangerousGetHandle(), bytes, (UIntPtr)bytes.Length), "read object field failed");
	}
	internal static bool HasField(ValueHandle handle, string key)
	{
		var bytes = Encoding.UTF8.GetBytes(key);
		return NativeBindings.bindings_value_object_has(handle.DangerousGetHandle(), bytes, (UIntPtr)bytes.Length);
	}
	internal static void PutField(ValueHandle handle, string key, ValueHandle value)
	{
		var bytes = Encoding.UTF8.GetBytes(key);
		if (!NativeBindings.bindings_value_object_put(handle.DangerousGetHandle(), bytes, (UIntPtr)bytes.Length, value.DangerousGetHandle()))
		{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("write object field failed"));
		}
	}
"#,
    )?;
    writer.ln("")
}

fn write_native_value_lists(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static IReadOnlyList<T> AsList<T>(ValueHandle handle, Func<ValueHandle, T> read)
	{
		var len = checked((int)NativeBindings.bindings_value_array_len(handle.DangerousGetHandle()).ToUInt64());
		var output = new List<T>(len);
		for (var idx = 0; idx < len; idx++)
		{
			using var item = FromRaw(NativeBindings.bindings_value_array_get(handle.DangerousGetHandle(), (UIntPtr)idx), "read array item failed");
			output.Add(read(item));
		}
		return output;
	}
	internal static IReadOnlySet<T> AsHashSet<T>(ValueHandle handle, Func<ValueHandle, T> read) => new HashSet<T>(AsList(handle, read));
	internal static ValueHandle FromList<T>(IEnumerable<T> values, Func<T, ValueHandle> write)
	{
		var source = values as ICollection<T> ?? values.ToArray();
		var arr = NewArray(source.Count);
		try
		{
			foreach (var item in source)
			{
				using var native = write(item);
				if (!NativeBindings.bindings_value_array_push(arr.DangerousGetHandle(), native.DangerousGetHandle()))
				{
					throw new InvalidOperationException(BindingErrors.LastErrorOrDefault("write array item failed"));
				}
			}
			return arr;
		}
		catch
		{
			arr.Dispose();
			throw;
		}
	}
"#,
    )?;
    writer.ln("")
}

fn write_native_value_unsupported(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
	internal static T Unsupported<T>(ValueHandle handle) => throw new NotSupportedException($"Unsupported C# binding type {typeof(T).Name}");
	internal static object UnsupportedDictionary(ValueHandle handle) => throw new NotSupportedException("Dictionary conversion is not implemented for C# bindings yet");
	internal static object UnsupportedTuple(ValueHandle handle) => throw new NotSupportedException("Tuple conversion is not implemented for C# bindings yet");
"#,
    )
}

fn write_binding_bytes(writer: &mut SourceWriter) -> Result<(), Error> {
    writer.block(
        r#"
internal static class BindingBytes
{
	internal static byte[] TakeBytes(IntPtr ptr, UIntPtr outLen, string fallback)
	{
		if (ptr == IntPtr.Zero)
		{
			throw new InvalidOperationException(BindingErrors.LastErrorOrDefault(fallback));
		}

		try
		{
			checked
			{
				var len = (int)outLen.ToUInt64();
				var managed = new byte[len];
				if (len > 0)
				{
					Marshal.Copy(ptr, managed, 0, len);
				}
				return managed;
			}
		}
		finally
		{
			NativeBindings.bindings_bytes_free(ptr, outLen);
		}
	}
}
"#,
    )
}
