# WASM (Rust <-> JS)

The `wasm` feature adds direct Rust <-> JavaScript conversion for generated protocol types.

This is intended for `wasm-bindgen` targets (browser and other JS runtimes) where you want to work with protocol objects in JS without JSON as a transport layer.

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

## Quick Start (wasm-bindgen Module)

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

Build example (from the `e2e/wasm_browser` workspace):

```bash
cd e2e/wasm_browser/bindings
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

- WASM browser e2e workspace: `e2e/wasm_browser/`
- Protocol crate: `e2e/wasm_browser/protocol`
- Bindings crate: `e2e/wasm_browser/bindings`
- Browser client: `e2e/wasm_browser/client`
- End-to-end script: `e2e/wasm_browser/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e/wasm_browser>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm_browser/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm_browser/bindings/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm_browser/client/src/main.js>
- <https://github.com/icsmw/brec/blob/main/e2e/wasm_browser/test.sh>

## Required Macros For Payload Types

For payload WASM conversion, nested custom Rust types must implement `brec::WasmConvert`.

Use:

- `#[derive(brec::Wasm)]` for nested structs/enums used inside payload fields
- `#[payload(bincode)]` for payloads supported by the generated Payload WASM aggregator

Example:

```rust
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

Typical browser-side flow:

```js
import init, { decode_packet, encode_packet } from 'wasmjs';

await init();

const packet = decode_packet(inBytes); // JS object
const outBytes = encode_packet(packet); // Uint8Array-compatible bytes
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
