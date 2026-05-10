# WASM (Rust <-> JS)

The `wasm` feature adds direct Rust <-> JavaScript conversion for generated protocol types.

This is intended for `wasm-bindgen` targets (browser and other JS runtimes) where you want to work with protocol objects in JS without JSON as a transport layer.
The wasm layer is a binding over the Rust packet engine, not a separate implementation of packet codecs in JavaScript. For the shared architectural model behind this split, see [Integrations](index.md).

## Motivation

The main reason to use `wasm` is to avoid extra conversion layers such as:

1. Rust binary -> Rust struct -> JSON string
2. JSON string -> JS object

and then the reverse on encode.

With `wasm`, conversion is done directly between Rust values and JS values (`JsValue`):

- less CPU spent on JSON serialization/parsing glue
- fewer temporary allocations related to string conversion
- strict numeric mapping for wide integers and float bit-exact roundtrips

As with any optimization, exact speedups depend on workload.
For packet-heavy wasm integrations, this removes a meaningful class of overhead.

## Enabling The Feature

Enable `wasm` in your protocol crate:

```toml
[dependencies]
brec = { version = "...", features = ["wasm", "bincode"] }
```

`bincode` is typically used because payload support in generated WASM aggregators expects payload variants to be `#[payload(bincode)]`.

## Quick Start (Generated Npm Package)

For most JavaScript integrations, use `brec_wasm_cli`. It generates both the `wasm-bindgen` Rust bindings crate and the TypeScript npm package from `brec.scheme.json`.

### 1. Export A Protocol Scheme

The CLI reads `brec.scheme.json`, so the protocol crate must explicitly enable scheme generation:

```rust
brec::generate!(scheme);
```

A plain `brec::generate!()` call does not write `brec.scheme.json`.

Custom Rust types used inside payload fields must also be exported into the scheme:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Wasm)]
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
cargo install brec_wasm_cli
```

After installation:

```bash
brec_wasm_cli --help
```

### 3. Generate The WASM Package

Node.js target:

```bash
brec_wasm_cli \
  --target node \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --npm-out path/to/generated/npm
```

Browser target:

```bash
brec_wasm_cli \
  --target browser \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --npm-out path/to/generated/npm
```

The Node.js package can be imported without explicit WASM initialization:

```ts
import { decodePacket, encodePacket } from "protocol";

const packet = decodePacket(bytes);
const encoded = encodePacket(packet);
```

The browser package exports `initWasm`; call it before using encode/decode functions:

```ts
import { decodePacket, encodePacket, initWasm } from "protocol";

await initWasm();
const packet = decodePacket(bytes);
const encoded = encodePacket(packet);
```

### CLI Options

`--target node|browser`

Required. Selects the JavaScript runtime target.

`node` calls `wasm-pack build --target nodejs` and generates a CommonJS `index.ts` entry point. Use it for Node.js clients.

`browser` calls `wasm-pack build --target web` and generates an ESM `index.ts` entry point that imports wasm-pack's async initializer and re-exports it as `initWasm`. Use it for browser clients and bundlers.

`--scheme <PATH>`

Path to `brec.scheme.json`. This file is emitted only when the protocol crate calls `brec::generate!(scheme)` and is built or checked. If omitted, the CLI searches from the current directory: first `./target/brec.scheme.json`, then recursively under the working directory.

`--protocol <DIR>`

Path to the Rust protocol crate used as the `protocol` dependency of the generated bindings crate. If omitted, the CLI infers it from the scheme path. For `target/brec.scheme.json`, the protocol directory is the parent of `target`; otherwise it is the scheme file directory.

`--bindings-out <DIR>`

Output directory for the generated Rust `wasm-bindgen` bindings crate. Defaults to `bindings` next to the scheme file.

`--out <DIR>`

Output directory for the generated npm package. Defaults to `npm` next to the scheme file.

`--npm-out <DIR>`

Alias for `--out`.

`--cargo-deps <PATH>`

Optional TOML file that overrides Cargo dependencies for the generated bindings crate. Most users do not need this option; it is mainly for local development and repository tests where the generated crate must link to local Rust crates instead of published versions.

`--npm-deps <PATH>`

Optional TOML file that overrides npm dependencies for the generated package. Most users do not need this option; it is mainly for local development and repository tests where the generated package must link to local npm packages instead of registry versions.

`-h`, `--help`

Prints CLI usage.

## Manual Quick Start (wasm-bindgen Module)

If you want to expose your protocol as a wasm module and use protocol objects directly in JS:

1. In your protocol crate, enable `brec` with `wasm` and your payload codec (usually `bincode`).
2. Define blocks with `#[brec::block]`.
3. Define payloads with `#[payload(bincode)]`.
4. For nested custom payload field types, derive `brec::Wasm`.
5. Call `brec::generate!()` to generate `Block`, `Payload`, and `Packet` glue.
6. In your wasm bindings crate, expose functions with `#[wasm_bindgen]` and call generated helpers:
   - `Block::decode_wasm` / `Block::encode_wasm`
   - `Payload::decode_wasm` / `Payload::encode_wasm`
   - `Packet::decode_wasm` / `Packet::encode_wasm`
7. Build the bindings crate with `wasm-pack` (or your preferred wasm-bindgen workflow) and consume it from JS.

Build example (from the `e2e/wasm` workspace):

```bash
cd e2e/wasm/binding
wasm-pack build --dev --target web --out-dir pkg --out-name wasmjs
```

Minimal shape:

```rust
// protocol crate
#[brec::block]
pub struct MyBlock {
    pub id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, brec::Wasm)]
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
#[wasm_bindgen]
pub fn decode_packet(buf: &[u8]) -> Result<JsValue, JsValue> {
    let mut ctx = ();
    Packet::decode_wasm(buf, &mut ctx)
        .map_err(|e| JsValue::from_str(&format!("decode packet: {e}")))
}

#[wasm_bindgen]
pub fn encode_packet(packet: JsValue) -> Result<Vec<u8>, JsValue> {
    let mut ctx = ();
    let mut out = Vec::new();
    Packet::encode_wasm(packet, &mut out, &mut ctx)
        .map_err(|e| JsValue::from_str(&format!("encode packet: {e}")))?;
    Ok(out)
}
```

Reference implementation in this repository:

- WASM shared e2e workspace: `e2e/wasm/`
- Protocol crate: `e2e/wasm/protocol`
- Binding crate: `e2e/wasm/binding`
- Browser client: `e2e/wasm/clients/browser`
- Node client: `e2e/wasm/clients/node`
- Generated package e2e workspace: `e2e-generator/wasm/`
- End-to-end scripts: `e2e/wasm/clients/browser/test.sh`, `e2e/wasm/clients/node/test.sh`, `e2e/wasm/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e/wasm>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm/binding/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm/clients/browser/src/main.js>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm/clients/node/src/main.js>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm/test.sh>
- <https://github.com/icsmw/brec/tree/main/e2e-generator/wasm>
- <https://github.com/icsmw/brec/blob/main/e2e-generator/wasm/clients/browser/src/main.ts>
- <https://github.com/icsmw/brec/blob/main/e2e-generator/wasm/clients/node/src/main.ts>

## Required Macros For Payload Types

For payload WASM conversion, nested custom Rust types must implement `brec::WasmConvert`.
For CLI-generated TypeScript declarations, those same nested types must also be present in `brec.scheme.json`.

Use:

- `#[derive(brec::Wasm)]` for nested structs/enums used inside payload fields
- `#[payload(include)]` for nested structs/enums that should be exported into `scheme.types`
- `#[payload(bincode)]` for payloads supported by the generated Payload WASM aggregator

Example:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Wasm, Clone, Debug)]
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

If a payload variant is not `#[payload(bincode)]`, the generated WASM payload aggregator returns an error for that variant.
If a nested custom type is used in a payload field but is not marked with `#[payload(include)]`, `brec_wasm_cli` cannot emit the matching TypeScript declaration and fails with a missing included type error.

## Rust -> JS Reflection

The generated WASM API uses explicit object shapes.

### Block enum shape

Each block is represented as an object with exactly one key:

```js
{ "MyBlock": { /* block fields */ } }
```

### Payload enum shape

Each payload is represented as an object with exactly one key:

```js
{ "MyPayload": { /* payload fields */ } }
```

Default payloads (when enabled) are:

```js
{ "Bytes": [/* u8 array */] }
{ "String": "..." }
```

### Packet shape

`PacketDef` WASM conversion uses:

```js
{
  blocks: Array<object>,   // each element is one-key Block object
  payload: object | null   // one-key Payload object, null, or undefined on input
}
```

## Data Contract On The Consumer Side

On the JavaScript side you receive plain runtime values (`object`, `Array`, `BigInt`, `string`, etc.), not generated runtime types.

What `brec` guarantees:

- The decoded object shape follows the protocol definition.
- If a variant is `BlockA`, the object contains exactly `BlockA` fields.
- If a variant is `PayloadA`, the object contains exactly `PayloadA` fields.
- Field names are preserved exactly as defined in your Rust protocol types.

What `brec` does not do for you:

- It does not generate runtime validators in JS.
- It does not validate your application-level invariants in JS.

Responsibility split:

- `brec` validates protocol data while decoding and produces protocol-shaped objects.
- The generated npm package provides TypeScript declarations for protocol-shaped values.
- Your application is responsible for additional business-level validation.

How to read these objects in JS:

```js
const packet = decode_packet(bytes);

for (const blockObj of packet.blocks) {
  const [blockKind, blockFields] = Object.entries(blockObj)[0];
  // blockKind -> "BlockA", blockFields -> { ...fields from protocol... }
}

if (packet.payload != null) {
  const [payloadKind, payloadFields] = Object.entries(packet.payload)[0];
  // payloadKind -> "PayloadA", payloadFields -> { ...fields from protocol... }
}
```

## Numeric Mapping Rules

To keep conversion lossless:

- `i64`, `u64`, `i128`, `u128` are mapped via JS `BigInt`
- `f32` is transferred as its `u32` bit pattern
- `f64` is transferred as its `u64` bit pattern via JS `BigInt`

This preserves exact Rust values across JS roundtrips, including edge cases.

### Why Float Bit Patterns?

`f32`/`f64` are encoded via bit patterns rather than plain JS `Number` to avoid accidental precision loss and to preserve exact payload values end-to-end.

This is especially important when values are serialized/deserialized many times across runtime boundaries.

## Generated Helpers

Generated protocol types expose WASM helper methods:

- `decode_wasm(...)` - bytes -> JS object
- `encode_wasm(...)` - JS object -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

## JavaScript Usage Pattern

Typical browser-side flow when using `wasm-pack` output directly:

```js
import init, { decode_packet, encode_packet } from 'wasmjs';

await init();

const packet = decode_packet(inBytes); // JS object
const outBytes = encode_packet(packet); // Uint8Array-compatible bytes
```

With `brec_wasm_cli --target browser`, use the generated wrapper instead:

```ts
import { decodePacket, encodePacket, initWasm } from "protocol";

await initWasm();

const packet = decodePacket(inBytes);
const outBytes = encodePacket(packet);
```

With `brec_wasm_cli --target node`, no initialization call is required:

```ts
import { decodePacket, encodePacket } from "protocol";

const packet = decodePacket(inBytes);
const outBytes = encodePacket(packet);
```

For large integer fields (`i64/u64/i128/u128`), provide `BigInt` values from JS:

```js
const payload = {
  PayloadA: {
    field_u64: 42n,
    field_i128: -123n,
  },
};
```

## Error Behavior

WASM conversion errors are surfaced as conversion/shape errors (for example: invalid object shape, missing field, invalid field type/range).

Common causes:

- enum wrapper object has zero or multiple keys
- `BigInt`-required field receives `Number`
- tuple/array field shape mismatch
- payload variant not marked with `#[payload(bincode)]`

## Runtime Notes

- Browser wasm modules are typically built with `wasm-pack --target web`.
- In Node runtimes with wasm-bindgen, the same object conversion rules apply.
- The conversion API is independent from transport; you can use WebSocket, fetch, SharedArrayBuffer pipelines, etc.

## Limitations

- Source-based Rust coverage (`cargo llvm-cov`) for `wasm32-unknown-unknown` is not generally available in standard setups.
- If you need coverage in browser tests, treat JS-side coverage and Rust-native coverage as separate pipelines.

## See Also

- [Blocks](../parts/blocks.md)
- [Payloads](../parts/payloads.md)
- [Packets](../parts/packets.md)
- [NAPI (Rust <-> JS)](./napi.md)
- [Java (Rust <-> Java)](./java.md)
- [C# (Rust <-> C#)](./csharp.md)
