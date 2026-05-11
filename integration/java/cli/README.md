# Brec Java Bindings Generator

`brec_java_cli` generates a JNI bindings crate and Java source files for a Brec protocol.

The CLI reads `brec.scheme.json`, writes a small Rust `cdylib` crate that exposes `decode*` and `encode*` JNI functions, builds the native library, and writes Java sources that load that library.

## Requirements

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

The generated Java package currently uses `com.icsmw.brec` and contains:

- `ClientBindings.java` with native `decodeBlock`, `encodeBlock`, `decodePayload`, `encodePayload`, `decodePacket`, and `encodePacket`;
- `Packet.java`, a small `HashMap<String, Object>` packet wrapper;
- `Blocks.java`, block field wrappers and tagged block helpers;
- `Payloads.java`, payload field wrappers and tagged payload helpers;
- `native/libbindings.so`, `native/libbindings.dylib`, or `native/bindings.dll`, depending on the host platform.

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

Optional TOML file that overrides Cargo dependencies for the generated JNI crate. Most users do not need this; it is primarily for local development and repository tests.

`-h`, `--help`

Prints CLI usage.

## Runtime Shape

The Java integration uses plain Java runtime containers compatible with the existing `brec` Java conversion layer:

- blocks and payloads are tagged `Map<String, Object>` values;
- packets are maps with `blocks` and `payload` keys;
- `u64`, `i64`, `u128`, `i128`, and `f64` use `BigInteger`;
- smaller integer fields and `f32` bit patterns use `Long`;
- vectors use `List<T>`;
- fixed blobs use `byte[]`.

The generated wrappers extend or create these container shapes so values can be passed directly to `ClientBindings.encodePacket`.
