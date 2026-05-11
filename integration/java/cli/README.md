# Java bindings generator for Brec

`brec_java_cli` generates a JNI bindings crate and typed Java source files for a Brec protocol.

The CLI reads `brec.scheme.json`, writes a small Rust `cdylib` crate that exposes JNI `decode*` and `encode*` functions, builds the native library, and writes Java sources that load that library through a `Client` facade.

## Requirements

- A protocol crate with the `java` feature enabled.
- A protocol crate that calls `brec::generate!(scheme)`.
- `cargo` in `PATH`.
- `javac` in `PATH`.
- Custom payload field types marked with `#[payload(include)]`.
- Nested custom payload field types deriving `brec::Java`.

Example protocol setup:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Java)]
pub struct Inner {
    pub value: String,
}

#[payload(bincode)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyPayload {
    pub inner: Inner,
}

brec::generate!(scheme);
```

Run Cargo for the protocol crate first so the scheme is written:

```bash
cargo check -p your_protocol_crate
```

## Installation

```bash
cargo install brec_java_cli
```

## Usage

```bash
brec_java_cli \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --java-out path/to/generated/java
```

The generated Java package uses `com.icsmw.brec`:

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

The public API is typed:

```java
Packet packet = Client.decodePacket(bytes);
byte[] encoded = Client.encodePacket(packet);
```

## Options

`--scheme <PATH>`

Path to `brec.scheme.json`. If omitted, the CLI searches from the current directory.

`--protocol <DIR>`

Path to the Rust protocol crate used as the `protocol` dependency of the generated JNI crate. If omitted, the CLI infers it from the scheme path.

`--bindings-out <DIR>`

Output directory for the generated Rust JNI crate. Defaults to `bindings` next to the scheme file.

`--out <DIR>`

Output directory for generated Java sources and the native library. Defaults to `java` next to the scheme file.

`--java-out <DIR>`

Alias for `--out`.

`--cargo-deps <PATH>`

Optional TOML file that overrides Cargo dependencies for the generated JNI crate. Most users do not need this; it is mainly for local development and repository tests where the generated crate must link to local Rust crates instead of published versions.

`-h`, `--help`

Prints CLI usage.

## Generated Java shape

`Client` exposes typed encode/decode methods:

```java
Block block = Client.decodeBlock(bytes);
byte[] blockBytes = Client.encodeBlock(block);

Payload payload = Client.decodePayload(bytes);
byte[] payloadBytes = Client.encodePayload(payload);

Packet packet = Client.decodePacket(bytes);
byte[] packetBytes = Client.encodePacket(packet);
```

Blocks and payloads are generated as regular Java classes with public fields:

```java
public final class PayloadA implements Payload {
    public Long field_u8;
    public String field_str;
}
```

`Packet` contains typed block and payload references:

```java
public final class Packet {
    public List<Block> blocks;
    public Payload payload;
}
```

Some generated helper methods use `@SuppressWarnings("unchecked")`. This is intentional: the JNI bridge transfers nested values through generic Java containers, and Java type erasure does not let the compiler prove every restored `List<T>` or nested custom type cast. The suppression is kept inside generated conversion helpers; the public API remains typed and should be used through `Client`, `Packet`, `Block`, `Payload`, and the generated classes.

The generated native library is copied to `native/libbindings.so`, `native/libbindings.dylib`, or `native/bindings.dll`, depending on the host platform.

## Numeric mapping

To keep conversion lossless:

- `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, and `f32` bit patterns use `Long`;
- `u64`, `i64`, `u128`, `i128`, and `f64` bit patterns use `BigInteger`;
- vectors use `List<T>`;
- fixed blobs use `byte[]`.
