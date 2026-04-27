# NAPI (Rust <-> JS)

The `napi` feature adds direct Rust <-> JavaScript conversion for generated protocol types.

This is intended for Node.js bindings where you want to work with protocol objects in JS without going through JSON payload conversion.
The integration exposes a JavaScript-facing binding surface, while packet reading, writing, validation, and payload codecs remain in the Rust core. For the shared architectural model behind this split, see [Integrations](index.md).

## Motivation

The main reason to use `napi` is to avoid extra conversion layers such as:

1. Rust binary -> Rust struct -> JSON string
2. JSON string -> JS object

and then the reverse on encode.

With `napi`, conversion is done directly between Rust values and JS values through N-API:

- less CPU spent on serialization/parsing glue
- fewer temporary allocations related to JSON strings
- strict numeric mapping (especially for large integers and float edge cases)

This does not mean every workload is always faster, but for packet-heavy Node integrations it removes a meaningful class of overhead.

## Enabling The Feature

Enable `napi` in your protocol crate:

```toml
[dependencies]
brec = { version = "...", features = ["napi", "bincode"] }
```

`bincode` is typically used because payload support in the generated NAPI aggregators expects payload variants to be `#[payload(bincode)]`.

## Quick Start (Node Module)

If you want to expose your protocol as a `.node` module and use protocol objects directly in JS:

1. In your protocol crate, enable `brec` with `napi` and your payload codec (usually `bincode`).
2. Define blocks with `#[brec::block]`.
3. Define payloads with `#[payload(bincode)]`.
4. For nested custom payload field types, derive `brec::Napi`.
5. Call `brec::generate!()` to generate `Block`, `Payload`, and `Packet` glue.
6. In your bindings crate, expose Node functions with `#[napi]` and call generated helpers:
   - `Block::decode_napi` / `Block::encode_napi`
   - `Payload::decode_napi` / `Payload::encode_napi`
   - `Packet::decode_napi` / `Packet::encode_napi`
7. Build your bindings crate as `cdylib`, then load the produced `.node` module from Node.js.

Build example (from the `e2e/node` workspace):

```bash
cargo build -p bindings --release
```

Then copy/rename the built dynamic library to `bindings.node` for Node runtime loading.
In the e2e example this is done in `e2e/node/test.sh`.

Minimal shape:

```rust
// protocol crate
#[brec::block]
pub struct MyBlock {
    pub id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, brec::Napi)]
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
#[napi]
pub fn decode_packet<'env>(env: &'env napi::Env, buf: napi::bindgen_prelude::Buffer)
    -> napi::Result<napi::Unknown<'env>>
{
    let mut ctx = ();
    Packet::decode_napi(env, buf, &mut ctx)
        .map_err(|e| napi::Error::from_reason(format!("decode packet: {e}")))
}

#[napi]
pub fn encode_packet(
    env: napi::Env,
    packet: napi::Unknown<'_>,
) -> napi::Result<napi::bindgen_prelude::Buffer> {
    let mut ctx = ();
    let mut out = Vec::new();
    Packet::encode_napi(&env, packet, &mut out, &mut ctx)
        .map_err(|e| napi::Error::from_reason(format!("encode packet: {e}")))?;
    Ok(out.into())
}
```

Reference implementation in this repository:

- Node e2e workspace: `e2e/node/`
- Protocol crate: `e2e/node/protocol`
- Bindings crate: `e2e/node/bindings`
- Node client: `e2e/node/client`
- End-to-end script: `e2e/node/test.sh`

Direct links:

- <https://github.com/icsmw/brec/tree/main/e2e/node>
- <https://github.com/icsmw/brec/blob/main/e2e/node/protocol/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/node/bindings/src/lib.rs>
- <https://github.com/icsmw/brec/blob/main/e2e/node/client/src/main.js>
- <https://github.com/icsmw/brec/blob/main/e2e/node/test.sh>

## Required Macros For Payload Types

For payload NAPI conversion, nested custom Rust types must implement `brec::NapiConvert`.

Use:

- `#[derive(brec::Napi)]` for nested structs/enums used inside payload fields
- `#[payload(bincode)]` for payloads that should be supported by the generated Payload NAPI aggregator

Example:

```rust
#[derive(serde::Serialize, serde::Deserialize, brec::Napi, Clone, Debug)]
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

If a payload variant is not `#[payload(bincode)]`, the generated NAPI payload aggregator returns an error for that variant.

## Rust -> JS Reflection

The generated NAPI API uses explicit object shapes.

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

`PacketDef` NAPI conversion uses:

```js
{
  blocks: Array<object>,   // each element is one-key Block object
  payload: object | null   // one-key Payload object, null, or undefined on input
}
```

## Data Contract On The Consumer Side

On the Node.js side you receive plain runtime JavaScript values (`object`, `Array`, `BigInt`, `string`, etc.), not generated runtime types.

What `brec` guarantees:

- The decoded object shape follows the protocol definition.
- If a variant is `BlockA`, the object contains exactly `BlockA` fields.
- If a variant is `PayloadA`, the object contains exactly `PayloadA` fields.
- Field names are preserved exactly as defined in your Rust protocol types.

What `brec` does not do for you:

- It does not generate runtime validators in JS.
- It does not enforce TypeScript compile-time typing by itself.

Responsibility split:

- `brec` validates protocol data while decoding and produces protocol-shaped objects.
- Your application is responsible for additional business-level validation and optional static typing wrappers.

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

This is deliberate: it preserves exact Rust values across JS roundtrips, including edge cases.

## Generated Helpers

Generated protocol types expose NAPI helper methods:

- `decode_napi(...)` - bytes -> JS object
- `encode_napi(...)` - JS object -> bytes

For packet and payload paths, context is passed explicitly (`ctx`) exactly like in regular Rust encode/decode flows.

## See Also

- [WASM (Rust <-> JS)](./wasm.md)
- [Java (Rust <-> Java)](./java.md)
