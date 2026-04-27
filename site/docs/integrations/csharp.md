# C# (Rust <-> C#)

The `csharp` feature adds direct Rust-side conversion support for C#-facing integrations over generated protocol types.

This is intended for C ABI / PInvoke-based integrations where you want to avoid JSON as an intermediate layer.
The C# layer is still a binding over the Rust packet engine, not a separate .NET-side reimplementation of packet codecs. For the shared architectural model behind this split, see [Integrations](index.md).

## Motivation

The main reason to use `csharp` is to avoid extra conversion layers such as:

1. Rust binary -> Rust struct -> JSON string
2. JSON string -> C# object

and then the reverse on encode.

With `csharp`, conversion is done against a schema-driven Rust-side value ABI (`CSharpValue`), which you can expose through a small FFI layer or keep behind opaque native handles:

- less CPU spent on serialization/parsing glue
- fewer temporary allocations related to JSON strings
- strict integer width preservation
- float bit-exact roundtrips

As with any optimization, exact speedups depend on workload.

## Enabling The Feature

Enable `csharp` in your protocol crate:

```toml
[dependencies]
brec = { version = "...", features = ["csharp", "bincode"] }
```

`bincode` is typically used because payload support in the generated C# aggregators expects payload variants to be `#[payload(bincode)]`.

## Quick Start (PInvoke Module)

If you want to expose your protocol to .NET through a native library:

1. In your protocol crate, enable `brec` with `csharp` and your payload codec (usually `bincode`).
2. Define blocks with `#[brec::block]`.
3. Define payloads with `#[payload(bincode)]`.
4. For nested custom payload field types, derive `brec::CSharp`.
5. Call `brec::generate!()` to generate `Block`, `Payload`, and `Packet` glue.
6. In your bindings crate, expose `extern "C"` functions and call generated helpers:
   - `Block::decode_csharp` / `Block::encode_csharp`
   - `Payload::decode_csharp` / `Payload::encode_csharp`
   - `Packet::decode_csharp` / `Packet::encode_csharp`
7. In C#, import those native functions with `DllImport`, plus any `free` / error-access helpers your FFI layer exposes, and build the managed API you want on top.

Minimal shape:

```rust
// protocol crate
#[brec::block]
pub struct MyBlock {
    pub id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, brec::CSharp)]
pub struct Inner {
    pub tag: String,
}

#[payload(bincode)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyPayload {
    pub inner: Inner,
}

brec::generate!();
```

```rust
// bindings crate
pub struct PacketHandle {
    value: brec::csharp_feat::CSharpValue,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_packet_decode(
    bytes_ptr: *const u8,
    bytes_len: usize,
) -> *mut PacketHandle {
    let bytes = if bytes_len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) }
    };

    let mut ctx = ();
    match Packet::decode_csharp(bytes, &mut ctx) {
        Ok(value) => Box::into_raw(Box::new(PacketHandle { value })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_packet_encode(
    handle: *const PacketHandle,
    out_len: *mut usize,
) -> *mut u8 {
    let handle_ref = unsafe { &*handle };
    let mut ctx = ();
    let mut out = Vec::new();
    if Packet::encode_csharp(handle_ref.value.clone(), &mut out, &mut ctx).is_err() {
        return std::ptr::null_mut();
    }
    let mut boxed = out.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    unsafe { *out_len = boxed.len() };
    std::mem::forget(boxed);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bindings_packet_free(handle: *mut PacketHandle) {
    if !handle.is_null() {
        unsafe { let _ = Box::from_raw(handle); }
    }
}
```

```csharp
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

internal static class Native
{
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
}
```

Reference implementation in this repository:

- C# e2e workspace: `e2e/csharp/`
- Protocol crate: `e2e/csharp/protocol`
- Bindings crate: `e2e/csharp/bindings`
- C# client: `e2e/csharp/client`
- End-to-end script: `e2e/csharp/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e/csharp>
- <https://github.com/icsmw/brec/blob/main/e2e/csharp/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/csharp/bindings/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/csharp/client/ClientBindings.cs>
- <https://github.com/icsmw/brec/blob/main/e2e/csharp/client/Program.cs>
- <https://github.com/icsmw/brec/blob/main/e2e/csharp/test.sh>

## Required Macros For Payload Types

For payload C# conversion, nested custom Rust types must implement `brec::CSharpConvert`.

Use:

- `#[derive(brec::CSharp)]` for nested structs/enums used inside payload fields
- `#[payload(bincode)]` for payloads supported by the generated Payload C# aggregator

Example:

```rust
#[derive(serde::Serialize, serde::Deserialize, brec::CSharp, Clone, Debug)]
pub struct Inner {
    pub id: u32,
    pub flag: bool,
}

#[payload(bincode)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MyPayload {
    pub inner: Inner,
}
```

If a payload variant is not `#[payload(bincode)]`, the generated C# payload aggregator returns an error for that variant.

## Rust -> C# Reflection

The generated C# integration uses explicit object shapes through the Rust-side `CSharpValue` ABI.

### Block enum shape

Each block is represented as an object with exactly one key:

```text
{ "MyBlock": { /* block fields */ } }
```

### Payload enum shape

Each payload is represented as an object with exactly one key:

```text
{ "MyPayload": { /* payload fields */ } }
```

Default payloads (when enabled) are:

```text
{ "Bytes": [/* u8 array */] }
{ "String": "..." }
```

### Packet shape

`PacketDef` C# conversion uses:

```text
{
  blocks: Array<object>,   // each element is one-key Block object
  payload: object | null   // one-key Payload object or null
}
```

## Data Contract On The Consumer Side

At the generated Rust API boundary, packet data is represented as `CSharpValue`, not as generated C# DTO classes.

What `brec` guarantees:

- The decoded object shape follows the protocol definition.
- If a variant is `BlockA`, the object contains exactly `BlockA` fields.
- If a variant is `PayloadA`, the object contains exactly `PayloadA` fields.
- Field names are preserved exactly as defined in your Rust protocol types.

What `brec` does not do for you:

- It does not generate managed C# classes automatically.
- It does not prescribe a single FFI transport strategy for `CSharpValue`.
- It does not perform your business-level validation.

Responsibility split:

- `brec` validates protocol data while decoding and produces protocol-shaped values.
- Your bindings layer decides how those values cross the native boundary.
- Your application is responsible for additional business-level validation and optional mapping to typed domain models.

The `e2e/csharp` example keeps decoded packets behind an opaque native handle and re-encodes them from Rust. It does not project `CSharpValue` into managed objects on the .NET side. If you want managed object projection, build it explicitly in your FFI layer around the same `CSharpValue` contract.

## Numeric Mapping Rules

To keep conversion lossless, the Rust-side C# ABI preserves exact scalar kinds:

- `u8`, `u16`, `u32`, `u64`, `u128` stay unsigned integers
- `i8`, `i16`, `i32`, `i64`, `i128` stay signed integers
- `f32` is transferred as its `u32` bit pattern
- `f64` is transferred as its `u64` bit pattern
- fixed byte arrays (`[u8; N]`) are transferred as bytes

This preserves exact Rust values across roundtrips, including float edge cases.

### Why Float Bit Patterns?

`f32`/`f64` are encoded via bit patterns rather than plain floating-point transport to avoid accidental precision loss and to preserve exact payload values end-to-end.

## Generated Helpers

Generated protocol types expose Rust-side C# integration helper methods:

- `decode_csharp(...)` - bytes -> `CSharpValue`
- `encode_csharp(...)` - `CSharpValue` -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

## Error Behavior

C# conversion errors from the generated helpers are surfaced as conversion/shape errors (for example: invalid object shape, missing field, invalid field type/range).

In the `e2e/csharp` bindings crate, those errors are flattened to strings and exposed through `bindings_last_error_message()`.

Common causes:

- enum wrapper object has zero or multiple keys
- integer value does not fit the target Rust type
- float field is not passed as the expected bit-pattern representation
- payload variant not marked with `#[payload(bincode)]`

## Runtime Notes

- The `csharp` feature gives you the Rust-side conversion contract, not a built-in .NET runtime package.
- In practice you will usually expose a `cdylib` and consume it through `DllImport` / PInvoke.
- The repository e2e client targets `.NET 8`, but the feature itself is not tied to that exact application shape.

## See Also

- [Blocks](../parts/blocks.md)
- [Payloads](../parts/payloads.md)
- [Packets](../parts/packets.md)
- [NAPI (Rust <-> JS)](./napi.md)
- [WASM (Rust <-> JS)](./wasm.md)
- [Java (Rust <-> Java)](./java.md)
