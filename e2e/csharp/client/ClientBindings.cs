using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace BrecE2e;

internal sealed class PacketHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public PacketHandle() : base(ownsHandle: true) {}

    public PacketHandle(IntPtr handlePtr) : base(ownsHandle: true)
    {
        SetHandle(handlePtr);
    }

    protected override bool ReleaseHandle()
    {
        ClientBindings.FreePacketHandle(handle);
        return true;
    }
}

internal static class ClientBindings
{
    private static class Native
    {
        [DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr bindings_last_error_message();

        [DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr bindings_packet_decode(
            [In] byte[] bytes,
            UIntPtr bytes_len);

        [DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr bindings_packet_encode(
            IntPtr handle,
            out UIntPtr out_len);

        [DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void bindings_packet_free(IntPtr handle);

        [DllImport("bindings", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void bindings_bytes_free(IntPtr ptr, UIntPtr len);
    }

    internal static void FreePacketHandle(IntPtr handle)
    {
        Native.bindings_packet_free(handle);
    }

    private static string LastErrorOrDefault(string fallback)
    {
        var ptr = Native.bindings_last_error_message();
        if (ptr == IntPtr.Zero)
        {
            return fallback;
        }
        return Marshal.PtrToStringAnsi(ptr) ?? fallback;
    }

    public static PacketHandle DecodePacket(byte[] bytes)
    {
        var ptr = Native.bindings_packet_decode(bytes, (UIntPtr)bytes.Length);
        if (ptr == IntPtr.Zero)
        {
            throw new InvalidOperationException(LastErrorOrDefault("decode packet failed"));
        }
        return new PacketHandle(ptr);
    }

    public static byte[] EncodePacket(PacketHandle packet)
    {
        if (packet.IsInvalid)
        {
            throw new InvalidOperationException("encode packet failed: invalid packet handle");
        }

        var ptr = Native.bindings_packet_encode(packet.DangerousGetHandle(), out var outLen);
        if (ptr == IntPtr.Zero)
        {
            throw new InvalidOperationException(LastErrorOrDefault("encode packet failed"));
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
            Native.bindings_bytes_free(ptr, outLen);
        }
    }
}
