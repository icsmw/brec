# C# (Rust <-> C#)

The `csharp` feature adds direct Rust <-> C# conversion support for generated protocol types.

This is intended for P/Invoke-based integrations where you want to work with protocol objects directly on the C# side without JSON as an intermediate layer.
The C# layer is a binding over the Rust packet engine, not a .NET-side reimplementation of packet codecs. For the shared architectural model behind this split, see [Integrations](index.md).

## Motivation

The main reason to use `csharp` is to avoid extra conversion layers such as:

1. Rust binary -> Rust struct -> JSON string
2. JSON string -> C# object

and then the reverse on encode.

With `csharp`, conversion is done through a schema-driven Rust-side value ABI (`CSharpValue`) and projected into generated C# classes:

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

`bincode` is typically used because payload support in generated C# aggregators expects payload variants to be `#[payload(bincode)]`.

## Quick Start (Generated C# Project)

For most C# integrations, use `brec_csharp_cli`. It generates both the native P/Invoke bindings crate and a C# project from `brec.scheme.json`.

### 1. Export A Protocol Scheme

The CLI reads `brec.scheme.json`, so the protocol crate must explicitly enable scheme generation:

```rust
brec::generate!(scheme);
```

A plain `brec::generate!()` call does not write `brec.scheme.json`.

Custom Rust types used inside payload fields must also be exported into the scheme:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::CSharp)]
pub struct Inner {
    pub tag: String,
}

#[payload(bincode)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyPayload {
    pub inner: Inner,
}

brec::generate!(scheme);
```

Run Cargo for the protocol crate so the macro writes the scheme file:

```bash
cargo check -p your_protocol_crate
```

By default, the scheme is written to `target/brec.scheme.json` for that crate.

### 2. Install The CLI

```bash
cargo install brec_csharp_cli
```

After installation:

```bash
brec_csharp_cli --help
```

### 3. Generate The C# Project

```bash
brec_csharp_cli \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --csharp-out path/to/generated/csharp
```

The generated C# API is typed:

```csharp
using Protocol;

Packet packet = PacketBindings.DecodePacket(bytes);
byte[] encoded = PacketBindings.EncodePacket(packet);
```

### CLI Options

`--scheme <PATH>`

Path to `brec.scheme.json`. This file is emitted only when the protocol crate calls `brec::generate!(scheme)` and is built or checked. If omitted, the CLI searches from the current directory: first `./target/brec.scheme.json`, then recursively under the working directory.

`--protocol <DIR>`

Path to the Rust protocol crate used as the `protocol` dependency of the generated native bindings crate. If omitted, the CLI infers it from the scheme path. For `target/brec.scheme.json`, the protocol directory is the parent of `target`; otherwise it is the scheme file directory.

`--bindings-out <DIR>`

Output directory for the generated Rust P/Invoke bindings crate. Defaults to `bindings` next to the scheme file.

`--out <DIR>`

Output directory for the generated C# project and native library. Defaults to `csharp` next to the scheme file.

`--csharp-out <DIR>`

Alias for `--out`.

`--cargo-deps <PATH>`

Optional TOML file that overrides Cargo dependencies for the generated bindings crate. Most users do not need this option; it is mainly for local development and repository tests where the generated crate must link to local Rust crates instead of published versions.

`-h`, `--help`

Prints CLI usage.

### Generated C# Project

The generated project is split by responsibility:

```text
Protocol.csproj
Bindings.cs
Blocks.cs
Payloads.cs
Packet.cs
native/<platform library>
```

`PacketBindings`, `BlockBindings`, and `PayloadBindings` are the public encode/decode facades:

```csharp
Block block = BlockBindings.DecodeBlock(bytes);
byte[] blockBytes = BlockBindings.EncodeBlock(block);

Payload payload = PayloadBindings.DecodePayload(bytes);
byte[] payloadBytes = PayloadBindings.EncodePayload(payload);

Packet packet = PacketBindings.DecodePacket(bytes);
byte[] packetBytes = PacketBindings.EncodePacket(packet);
```

`Packet` keeps typed references:

```csharp
public sealed class Packet
{
    public IReadOnlyList<Block> Blocks { get; }
    public Payload? Payload { get; }
}
```

Blocks and payloads are generated as immutable `abstract` / `sealed` class hierarchies:

```csharp
public abstract class Payload
{
    private protected Payload() { }
}

public sealed class PayloadA : Payload
{
    public byte FieldU8 { get; }
    public string FieldStr { get; }
}
```

Rust enums with data are generated as nested element classes because C# `enum` cannot carry per-variant payload values:

```csharp
public abstract class PayloadD : Payload
{
    public enum Kind
    {
        U8,
        String,
    }

    public abstract Kind Variant { get; }

    public sealed class ElementString : PayloadD
    {
        public string Value { get; }
    }
}
```

`Bindings.cs` contains internal P/Invoke declarations, `SafeHandle` wrappers, and conversion helpers. Public code should use `PacketBindings`, `BlockBindings`, `PayloadBindings`, `Packet`, `Block`, `Payload`, and the generated block/payload classes.

## Required Macros For Payload Types

For payload C# conversion, nested custom Rust types must implement `brec::CSharpConvert`.
For CLI-generated C# classes, those same nested types must also be present in `brec.scheme.json`.

Use:

- `#[derive(brec::CSharp)]` for nested structs/enums used inside payload fields
- `#[payload(include)]` for nested structs/enums that should be exported into `scheme.types`
- `#[payload(bincode)]` for payloads supported by the generated Payload C# aggregator

Example:

```rust
#[payload(include)]
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
If a nested custom type is used in a payload field but is not marked with `#[payload(include)]`, `brec_csharp_cli` cannot emit the matching C# class and fails with a missing included type error.

## Runtime Reflection Shape

Internally, the P/Invoke bridge passes explicit object/list shapes through `CSharpValue`. The generated C# classes hide that representation behind typed properties.

### Block enum shape

Each block is represented internally as an object with exactly one key:

```text
{ "MyBlock": { /* block fields object */ } }
```

### Payload enum shape

Each payload is represented internally as an object with exactly one key:

```text
{ "MyPayload": { /* payload fields object */ } }
```

Default payloads, when enabled, are:

```text
{ "Bytes": [/* u8 array */] }
{ "String": "..." }
```

### Packet shape

`PacketDef` C# conversion uses:

```text
{
  "blocks":  Array<object>, // each element is one-key Block object
  "payload": object | null  // one-key Payload object or null
}
```

Application code normally does not need to work with these shapes directly. Use generated classes and `Encode*` / `Decode*` facades instead.

## Numeric Mapping Rules

To keep conversion lossless:

- `u8`, `u16`, `u32`, `u64`, and `u128` map to `byte`, `ushort`, `uint`, `ulong`, and `UInt128`
- `i8`, `i16`, `i32`, `i64`, and `i128` map to `sbyte`, `short`, `int`, `long`, and `Int128`
- `f32` is transferred as its `u32` bit pattern and restored to `float`
- `f64` is transferred as its `u64` bit pattern and restored to `double`
- vectors use `IReadOnlyList<T>`
- fixed byte blobs use `byte[]`

This preserves exact Rust values across C# roundtrips, including float edge cases.

## Manual Quick Start (P/Invoke Library)

The generated CLI project is usually the easiest path. If you need a custom P/Invoke crate, enable the `csharp` feature and call the generated helpers yourself:

```rust
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
```

Generated protocol types expose Rust-side C# helper methods:

- `decode_csharp(...)` - bytes -> `CSharpValue`
- `encode_csharp(...)` - `CSharpValue` -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

### Data Contract On The Consumer Side

If you use the Rust helpers manually, the Rust API boundary is `CSharpValue`, not generated C# classes. Your FFI layer decides how to expose that value tree to .NET.

What `brec` guarantees:

- The decoded object shape follows the protocol definition.
- If a variant is `BlockA`, the object contains exactly `BlockA` fields.
- If a variant is `PayloadA`, the object contains exactly `PayloadA` fields.
- Field names are preserved exactly as defined in your Rust protocol types.

What `brec` does not do for a manual P/Invoke layer:

- It does not prescribe a single native transport strategy for `CSharpValue`.
- It does not perform your business-level validation.

Responsibility split:

- `brec` validates protocol data while decoding and produces protocol-shaped values.
- Your bindings layer decides how those values cross the native boundary.
- Your application is responsible for additional validation and optional mapping to typed domain models.

Reference implementation in this repository:

- Generated C# e2e workspace: `e2e-gen/csharp/`
- Protocol crate: `e2e-gen/csharp/protocol`
- C# client: `e2e-gen/csharp/client`
- End-to-end script: `e2e-gen/csharp/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e-gen/csharp>
- <https://github.com/icsmw/brec/blob/main/e2e-gen/csharp/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e-gen/csharp/client/Program.cs>
- <https://github.com/icsmw/brec/blob/main/e2e-gen/csharp/test.sh>

## See Also

- [Blocks](../parts/blocks.md)
- [Payloads](../parts/payloads.md)
- [Packets](../parts/packets.md)
- [NAPI (Rust <-> JS)](./napi.md)
- [WASM (Rust <-> JS)](./wasm.md)
- [Java (Rust <-> Java)](./java.md)
