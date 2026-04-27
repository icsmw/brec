# Java (Rust <-> Java)

The `java` feature adds direct Rust <-> Java conversion for generated protocol types.

This is intended for JNI-based integrations where you want to work with protocol objects directly on the Java side without JSON as an intermediate layer.
The Java layer is a JNI-facing binding over the Rust packet engine, not a Java-side reimplementation of packet codecs. For the shared architectural model behind this split, see [Integrations](index.md).

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

## Quick Start (JNI Module)

If you want to expose your protocol as a JNI library and use protocol objects directly in Java:

1. In your protocol crate, enable `brec` with `java` and your payload codec (usually `bincode`).
2. Define blocks with `#[brec::block]`.
3. Define payloads with `#[payload(bincode)]`.
4. For nested custom payload field types, derive `brec::Java`.
5. Call `brec::generate!()` to generate `Block`, `Payload`, and `Packet` glue.
6. In your JNI bindings crate, expose native methods and call generated helpers:
   - `Block::decode_java` / `Block::encode_java`
   - `Payload::decode_java` / `Payload::encode_java`
   - `Packet::decode_java` / `Packet::encode_java`

Minimal shape:

```rust
// protocol crate
#[brec::block]
pub struct MyBlock {
    pub id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, brec::Java)]
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
// bindings crate (JNI exports)
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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_Bindings_encodePacket<'local>(
    mut env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    packet: jni::objects::JObject<'local>,
) -> jni::sys::jbyteArray {
    let mut ctx = ();
    let mut out = Vec::new();
    Packet::encode_java(&mut env, packet, &mut out, &mut ctx).expect("encode");
    env.byte_array_from_slice(&out).expect("alloc").into_raw()
}
```

Reference implementation in this repository:

- Java e2e workspace: `e2e/java/`
- Protocol crate: `e2e/java/protocol`
- Bindings crate: `e2e/java/bindings`
- Java client: `e2e/java/client`
- End-to-end script: `e2e/java/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e/java>
- <https://github.com/icsmw/brec/blob/main/e2e/java/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/java/bindings/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/java/client/src/com/icsmw/brec/Main.java>
- <https://github.com/icsmw/brec/blob/main/e2e/java/test.sh>

## Required Macros For Payload Types

For payload Java conversion, nested custom Rust types must implement `brec::JavaConvert`.

Use:

- `#[derive(brec::Java)]` for nested structs/enums used inside payload fields
- `#[payload(bincode)]` for payloads supported by the generated Payload Java aggregator

If a payload variant is not `#[payload(bincode)]`, the generated Java payload aggregator returns an error for that variant.

## Rust -> Java Reflection

The generated Java API uses explicit object shapes implemented with `java.util.HashMap` / `java.util.ArrayList`.

### Block enum shape

Each block is represented as a map with exactly one key:

```java
{ "MyBlock" -> { /* block fields map */ } }
```

### Payload enum shape

Each payload is represented as a map with exactly one key:

```java
{ "MyPayload" -> { /* payload fields map */ } }
```

Default payloads (when enabled) are:

```java
{ "Bytes" -> java.util.ArrayList<Long> }
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

## Data Contract On The Consumer Side

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

## Numeric Mapping Rules

To keep conversion lossless:

- `u8`, `u16`, `u32`, `i8`, `i16`, `i32` are mapped through `java.lang.Long` with range checks
- `i64`, `u64`, `i128`, `u128` are mapped via `java.math.BigInteger`
- `f32` is transferred as its `u32` bit pattern
- `f64` is transferred as its `u64` bit pattern via `BigInteger`

This preserves exact Rust values across Java roundtrips, including float edge cases.

## Generated Helpers

Generated protocol types expose Java helper methods:

- `decode_java(...)` - bytes -> Java object
- `encode_java(...)` - Java object -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

## See Also

- [Blocks](../parts/blocks.md)
- [Payloads](../parts/payloads.md)
- [Packets](../parts/packets.md)
- [NAPI (Rust <-> JS)](./napi.md)
- [WASM (Rust <-> JS)](./wasm.md)
