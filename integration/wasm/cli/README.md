# Brec WASM Bindings Generator

`brec_wasm_cli` is a command-line generator for packaging a Brec protocol as a ready-to-use WebAssembly npm package.

The crate reads `brec.scheme.json`, generates TypeScript declarations that mirror the Rust protocol model, creates a small `wasm-bindgen` bindings crate, builds it with `wasm-pack`, and writes an npm package that exposes typed `encode*` and `decode*` functions.

It is intended for projects that already define their protocol in Rust and want to consume binary Brec packets from JavaScript without maintaining a hand-written wasm bindings crate or a hand-written TypeScript type layer.

The main Brec WASM documentation is available at <https://icsmw.github.io/brec/integrations/wasm/>. Read it for protocol-side requirements, Rust-to-JavaScript value mapping, and the broader integration model.

## What It Generates

The CLI produces two outputs:

- a generated Rust wasm-bindgen crate, by default named `bindings`;
- a generated npm package, by default named `npm`.

The generated Rust crate depends on the protocol crate and exposes wasm functions for:

- `decodeBlock(bytes)` / `encodeBlock(block)`;
- `decodePayload(bytes)` / `encodePayload(payload)`;
- `decodePacket(bytes)` / `encodePacket(packet)`.

The generated npm package contains target-specific wasm-pack output plus TypeScript protocol declarations:

- `wasmjs.js` and `wasmjs_bg.wasm`;
- `index.ts`, the public TypeScript entry point;
- `blocks.ts`, generated block interfaces and the `Block` union;
- `payloads.ts`, included payload helper types, payload declarations, and the `Payload` union;
- `packet.ts`, the generated `Packet` interface;
- `package.json` and `tsconfig.json`.

After wasm-pack completes, the npm package is built with `npm install --package-lock=false` and `npm run build`.

## Requirements

The CLI expects:

- a protocol crate that generates `brec.scheme.json`;
- `cargo` available in `PATH`;
- `wasm-pack` available in `PATH`;
- `npm` available in `PATH`;
- Node dependencies resolvable from the generated npm package directory.

The protocol crate must explicitly enable scheme generation with `brec::generate!(scheme)`. A plain `brec::generate!()` call does not create `brec.scheme.json`, so this CLI has nothing to read.

Custom Rust types used inside payload fields must be exported into the scheme with `#[payload(include)]`. If such a type is referenced by a payload but not included, the CLI cannot generate the matching TypeScript declaration.

For WASM conversion, nested payload types also normally derive the required Brec conversion trait. See the main WASM documentation for the full protocol-side setup.

Minimal protocol-side shape:

```rust
#[payload(include)]
#[derive(serde::Serialize, serde::Deserialize, brec::Wasm)]
pub struct InnerPayloadType {
    pub value: String,
}

#[payload(bincode)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyPayload {
    pub inner: InnerPayloadType,
}

// The `scheme` flag is required for this CLI.
brec::generate!(scheme);
```

Build or check the protocol crate first so Cargo runs the macro and writes `target/brec.scheme.json`:

```bash
cargo check -p your_protocol_crate
```

## Installation

Install the CLI from crates.io:

```bash
cargo install brec_wasm_cli
```

After installation, the command is available as:

```bash
brec_wasm_cli --help
```

## Usage

```bash
brec_wasm_cli \
  --target node \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --npm-out path/to/generated/npm
```

If `--scheme` is omitted, the CLI searches for `brec.scheme.json` from the current working directory. It first checks `./target/brec.scheme.json`, then recursively scans the working directory. If more than one scheme file is found, the command fails and asks for an explicit `--scheme`.

`--target` is required. Use `--target node` for Node.js clients and `--target browser` for browser bundlers or browser-native ESM code.

Browser packages export `initWasm`, which must be awaited before calling the encode/decode functions:

```ts
import { decodePacket, encodePacket, initWasm } from "protocol";

await initWasm();
const packet = decodePacket(bytes);
const encoded = encodePacket(packet);
```

`--target node` generates a CommonJS entry point for Node.js and does not require an explicit initialization call.

## Options

`--scheme <PATH>`

Path to `brec.scheme.json`. This is the scheme file emitted by the protocol crate.

The file is emitted only when the protocol crate calls `brec::generate!(scheme)` and the crate is built or checked.

`--protocol <DIR>`

Path to the Rust protocol crate that should be used as the `protocol` dependency of the generated bindings crate. If omitted, the CLI infers it from the scheme path. For `target/brec.scheme.json`, the protocol directory is the parent of `target`; otherwise it is the scheme file directory.

`--target node|browser`

Required. Selects the JavaScript runtime target.

`node` calls `wasm-pack build --target nodejs` and generates a CommonJS `index.ts` entry point. Use it for Node.js clients.

`browser` calls `wasm-pack build --target web` and generates an ESM `index.ts` entry point that imports wasm-pack's async initializer and re-exports it as `initWasm`. Use it for browser clients and bundlers.

`--bindings-out <DIR>`

Output directory for the generated Rust wasm-bindgen bindings crate. Defaults to `bindings` next to the scheme file.

`--out <DIR>`

Output directory for the generated npm package. Defaults to `npm` next to the scheme file.

`--npm-out <DIR>`

Alias for `--out`.

`--cargo-deps <PATH>`

Optional TOML file that overrides Cargo dependencies for the generated bindings crate.

Most users do not need this option. It is mainly useful for local development and repository tests where the generated crate must use local Brec crates instead of published crate versions. Local dependency paths in this file are resolved relative to the override TOML file and then rewritten relative to `--bindings-out`.

`--npm-deps <PATH>`

Optional TOML file that overrides npm dependencies for the generated package.

Most users do not need this option. It is mainly useful for local development and repository tests where the generated package must link to local npm packages instead of registry versions. Local dependency paths in this file are resolved relative to the override TOML file and then rewritten as `file:` specs relative to the generated npm package directory.

`-h`, `--help`

Prints CLI usage.

## Dependency Overrides

Dependency override files are an advanced escape hatch. They are not part of the normal generator flow.

Use `--cargo-deps` when testing the CLI against local Rust crates:

```toml
[dependencies]
brec = { path = "../../lib/core", features = ["bincode"] }
```

Use `--npm-deps` when testing the generated package against local npm packages:

```toml
[dependencies]
some-package = { path = "../some-package" }
```

Registry versions can also be overridden, but production users should usually rely on the generator defaults.

## Output Contract

The generated package exposes plain JavaScript values with TypeScript declarations.

Blocks and payloads are represented as tagged objects:

```ts
{ SomeBlock: { /* block fields */ } }
{ SomePayload: { /* payload fields */ } }
```

Packets have this shape:

```ts
export interface Packet {
  blocks: Block[];
  payload?: Payload;
}
```

Large Rust integer types are represented as `bigint`; smaller numeric types are represented as `number`. Rust `Option<T>` becomes an optional property for named fields and `T | undefined` in tuple-like positions.
