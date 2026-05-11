# Java (Rust <-> Java)

The `java` feature adds direct Rust <-> Java conversion for generated protocol types.

This is intended for JNI-based integrations where you want to work with protocol objects directly on the Java side without JSON as an intermediate layer.
The Java layer is a binding over the Rust packet engine, not a Java-side reimplementation of packet codecs. For the shared architectural model behind this split, see [Integrations](index.md).

## Motivation

The main reason to use `java` is to avoid extra conversion layers such as:

1. Rust binary -> Rust struct -> JSON string
2. JSON string -> Java object

and then the reverse on encode.

With `java`, conversion is done directly between Rust values and Java objects through JNI:

- less CPU spent on serialization/parsing glue
- fewer temporary allocations related to JSON strings
- strict numeric mapping for wide integers and float bit-exact roundtrips

As with any optimization, exact speedups depend on workload.

## Enabling The Feature

Enable `java` in your protocol crate:

```toml
[dependencies]
brec = { version = "...", features = ["java", "bincode"] }
```

`bincode` is typically used because payload support in generated Java aggregators expects payload variants to be `#[payload(bincode)]`.

## Quick Start (Generated Java Package)

For most Java integrations, use `brec_java_cli`. It generates both the native JNI bindings crate and Java sources from `brec.scheme.json`.

### 1. Export A Protocol Scheme

The CLI reads `brec.scheme.json`, so the protocol crate must explicitly enable scheme generation:

```rust
brec::generate!(scheme);
```

A plain `brec::generate!()` call does not write `brec.scheme.json`.

Custom Rust types used inside payload fields must also be exported into the scheme:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Java)]
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
cargo install brec_java_cli
```

After installation:

```bash
brec_java_cli --help
```

### 3. Generate The Java Sources

```bash
brec_java_cli \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --java-out path/to/generated/java
```

The generated Java API is typed:

```java
import com.icsmw.brec.Client;
import com.icsmw.brec.Packet;

Packet packet = Client.decodePacket(bytes);
byte[] encoded = Client.encodePacket(packet);
```

### CLI Options

`--scheme <PATH>`

Path to `brec.scheme.json`. This file is emitted only when the protocol crate calls `brec::generate!(scheme)` and is built or checked. If omitted, the CLI searches from the current directory: first `./target/brec.scheme.json`, then recursively under the working directory.

`--protocol <DIR>`

Path to the Rust protocol crate used as the `protocol` dependency of the generated JNI bindings crate. If omitted, the CLI infers it from the scheme path. For `target/brec.scheme.json`, the protocol directory is the parent of `target`; otherwise it is the scheme file directory.

`--bindings-out <DIR>`

Output directory for the generated Rust JNI bindings crate. Defaults to `bindings` next to the scheme file.

`--out <DIR>`

Output directory for generated Java sources and the native library. Defaults to `java` next to the scheme file.

`--java-out <DIR>`

Alias for `--out`.

`--cargo-deps <PATH>`

Optional TOML file that overrides Cargo dependencies for the generated bindings crate. Most users do not need this option; it is mainly for local development and repository tests where the generated crate must link to local Rust crates instead of published versions.

`-h`, `--help`

Prints CLI usage.

### Generated Java Package

The generated package is split by responsibility:

```text
com.icsmw.brec
  Client
  Packet

com.icsmw.brec.block
  Block
  BlockSupport
  <generated block classes>

com.icsmw.brec.payload
  Payload
  PayloadSupport
  <generated payload and included helper classes>
```

`Client` is the public encode/decode facade:

```java
Block block = Client.decodeBlock(bytes);
byte[] blockBytes = Client.encodeBlock(block);

Payload payload = Client.decodePayload(bytes);
byte[] payloadBytes = Client.encodePayload(payload);

Packet packet = Client.decodePacket(bytes);
byte[] packetBytes = Client.encodePacket(packet);
```

`Packet` keeps typed references:

```java
public final class Packet {
    public List<Block> blocks;
    public Payload payload;
}
```

Blocks and payloads are generated as ordinary Java classes:

```java
package com.icsmw.brec.payload;

public final class PayloadA implements Payload {
    public Long field_u8;
    public String field_str;
}
```

`BlockSupport` and `PayloadSupport` are generated package-private helpers. Public code should use `Client`, `Packet`, `Block`, `Payload`, and the generated block/payload classes.

Some generated conversion helpers use `@SuppressWarnings("unchecked")`. This is intentional: the JNI bridge transfers nested values through generic Java containers, and Java type erasure does not let the compiler prove every restored `List<T>` or nested custom type cast. The suppression stays inside generated helper code; the public API remains typed.

## Required Macros For Payload Types

For payload Java conversion, nested custom Rust types must implement `brec::JavaConvert`.
For CLI-generated Java classes, those same nested types must also be present in `brec.scheme.json`.

Use:

- `#[derive(brec::Java)]` for nested structs/enums used inside payload fields
- `#[payload(include)]` for nested structs/enums that should be exported into `scheme.types`
- `#[payload(bincode)]` for payloads supported by the generated Payload Java aggregator

Example:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Java, Clone, Debug)]
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

If a payload variant is not `#[payload(bincode)]`, the generated Java payload aggregator returns an error for that variant.
If a nested custom type is used in a payload field but is not marked with `#[payload(include)]`, `brec_java_cli` cannot emit the matching Java class and fails with a missing included type error.

## Runtime Reflection Shape

Internally, the JNI bridge still passes explicit map/list shapes between Rust and Java. The generated Java classes hide that representation behind typed fields.

### Block enum shape

Each block is represented internally as a map with exactly one key:

```java
{ "MyBlock" -> { /* block fields map */ } }
```

### Payload enum shape

Each payload is represented internally as a map with exactly one key:

```java
{ "MyPayload" -> { /* payload fields map */ } }
```

Default payloads, when enabled, are:

```java
{ "Bytes" -> byte[] }
{ "String" -> java.lang.String }
```

### Packet shape

`PacketDef` Java conversion uses:

```java
{
  "blocks"  -> java.util.ArrayList<Object>, // each element is one-key Block map
  "payload" -> Object | null                 // one-key Payload map or null
}
```

Application code normally does not need to work with these maps directly. Use generated classes and `Client.encode*` / `Client.decode*` instead.

## Numeric Mapping Rules

To keep conversion lossless:

- `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, and `f32` bit patterns use `Long`
- `u64`, `i64`, `u128`, `i128`, and `f64` bit patterns use `BigInteger`
- vectors use `List<T>`
- fixed blobs use `byte[]`

This preserves exact Rust values across Java roundtrips, including float edge cases.

## Manual Quick Start (JNI Library)

The generated CLI package is usually the easiest path. If you need a custom JNI crate, enable the `java` feature and call the generated helpers yourself:

```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_Bindings_decodePacket<'local>(
    mut env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    bytes: jni::objects::JByteArray<'local>,
) -> jni::sys::jobject {
    let bytes = env.convert_byte_array(bytes).expect("bytes");
    let mut ctx = ();
    Packet::decode_java(&mut env, &bytes, &mut ctx)
        .expect("decode")
        .into_raw()
}
```

Generated protocol types expose Java helper methods:

- `decode_java(...)` - bytes -> Java object
- `encode_java(...)` - Java object -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

### Data Contract On The Consumer Side

On the Java side you receive runtime objects (`HashMap`, `ArrayList`, boxed primitives, `BigInteger`, `String`), not generated strongly typed DTO classes by default.

What `brec` guarantees:

- The decoded object shape follows the protocol definition.
- If a variant is `BlockA`, the object contains exactly `BlockA` fields.
- If a variant is `PayloadA`, the object contains exactly `PayloadA` fields.
- Field names are preserved exactly as defined in your Rust protocol types.

What `brec` does not do for you:

- It does not generate Java POJOs/records automatically.
- It does not perform your business-level validation.

Responsibility split:

- `brec` validates protocol data while decoding and produces protocol-shaped objects.
- Your application is responsible for additional validation and optional mapping to typed domain models.

How to read these objects in Java:

```java
Object raw = ClientBindings.decodePacket(bytes);
Map<?, ?> packet = (Map<?, ?>) raw;

List<?> blocks = (List<?>) packet.get("blocks");
for (Object item : blocks) {
    Map<?, ?> blockWrapper = (Map<?, ?>) item;
    Map.Entry<?, ?> entry = blockWrapper.entrySet().iterator().next();
    String blockKind = (String) entry.getKey();   // "BlockA"
    Map<?, ?> blockFields = (Map<?, ?>) entry.getValue();
}

Object payloadObj = packet.get("payload");
if (payloadObj != null) {
    Map<?, ?> payloadWrapper = (Map<?, ?>) payloadObj;
    Map.Entry<?, ?> entry = payloadWrapper.entrySet().iterator().next();
    String payloadKind = (String) entry.getKey(); // "PayloadA"
    Map<?, ?> payloadFields = (Map<?, ?>) entry.getValue();
}
```


Reference implementation in this repository:

- Generated Java e2e workspace: `e2e-generator/java/`
- Protocol crate: `e2e-generator/java/protocol`
- Java client: `e2e-generator/java/client`
- End-to-end script: `e2e-generator/java/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e-generator/java>
- <https://github.com/icsmw/brec/blob/main/e2e-generator/java/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e-generator/java/client/src/com/icsmw/brec/Main.java>
- <https://github.com/icsmw/brec/blob/main/e2e-generator/java/test.sh>

## See Also

- [Blocks](../parts/blocks.md)
- [Payloads](../parts/payloads.md)
- [Packets](../parts/packets.md)
- [NAPI (Rust <-> JS)](./napi.md)
- [WASM (Rust <-> JS)](./wasm.md)
- [C# (Rust <-> C#)](./csharp.md)
