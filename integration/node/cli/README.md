# Brec Node Bindings Generator

`brec_node_cli` is a command-line generator for packaging a Brec protocol as a ready-to-use Node.js npm package.

The crate reads `brec.scheme.json`, generates TypeScript declarations that mirror the Rust protocol model, builds a small native N-API bindings crate, and writes an npm package that exposes typed `encode*` and `decode*` functions.

It is intended for projects that already define their protocol in Rust and want to consume binary Brec packets directly from Node.js without maintaining a hand-written bindings crate or a hand-written TypeScript type layer.

The main Brec N-API documentation is available at <https://icsmw.github.io/brec/integrations/napi/>. Read it for protocol-side requirements, Rust-to-JavaScript value mapping, and the broader integration model.

## What It Generates

The CLI produces two outputs:

- a generated Rust bindings crate, by default named `bindings`;
- a generated npm package, by default named `npm`.

The generated Rust crate depends on the protocol crate and exposes native N-API functions for:

- `decodeBlock(bytes)` / `encodeBlock(block)`;
- `decodePayload(bytes)` / `encodePayload(payload)`;
- `decodePacket(bytes)` / `encodePacket(packet)`.

The generated npm package contains:

- `index.ts`, the public TypeScript entry point;
- `blocks.ts`, generated block interfaces and the `Block` union;
- `payloads.ts`, included payload helper types, payload declarations, and the `Payload` union;
- `packet.ts`, the generated `Packet` interface;
- `package.json` and `tsconfig.json`;
- `native/bindings.node`, copied from the generated Rust release build.

After generation, the npm package is built with `npm install --package-lock=false` and `npm run build`.

## Requirements

The CLI expects:

- a protocol crate that generated `brec.scheme.json`;
- `cargo` available in `PATH`;
- `npm` available in `PATH`;
- Node dependencies resolvable from the generated npm package directory.

The protocol crate normally enables Brec Node/N-API support through the relevant Brec features and derives required conversion traits for nested payload types. The e2e reference is `e2e-generator/node`.

## Installation

Install the CLI from crates.io:

```bash
cargo install brec_node_cli
```

After installation, the command is available as:

```bash
brec_node_cli --help
```

## Usage

```bash
brec_node_cli \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --npm-out path/to/generated/npm
```

If `--scheme` is omitted, the CLI searches for `brec.scheme.json` from the current working directory. It first checks `./target/brec.scheme.json`, then recursively scans the working directory. If more than one scheme file is found, the command fails and asks for an explicit `--scheme`.

## Options

`--scheme <PATH>`

Path to `brec.scheme.json`. This is the scheme file emitted by the protocol crate.

`--protocol <DIR>`

Path to the Rust protocol crate that should be used as the `protocol` dependency of the generated bindings crate. If omitted, the CLI infers it from the scheme path. For `target/brec.scheme.json`, the protocol directory is the parent of `target`; otherwise it is the scheme file directory.

`--bindings-out <DIR>`

Output directory for the generated Rust N-API bindings crate. Defaults to `bindings` next to the scheme file.

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

## Example From This Repository

The Node generator e2e workspace uses this CLI from `e2e-generator/node/test.sh`:

```bash
cargo run --manifest-path "${ROOT_DIR}/../../Cargo.toml" -p brec_node_cli -- \
  --scheme "${ROOT_DIR}/protocol/target/brec.scheme.json" \
  --protocol "${ROOT_DIR}/protocol" \
  --bindings-out "${ROOT_DIR}/bindings" \
  --npm-out "${ROOT_DIR}/generated/npm" \
  --cargo-deps "${ROOT_DIR}/local.deps.cargo.toml"
```

The generated npm package is then consumed by the Node client as a normal package and used like this:

```ts
import { decodePacket, encodePacket } from "protocol";

const packet = decodePacket(bytes);
const encoded = encodePacket(packet);
```

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

## Notes

This CLI owns only generated files it knows about. Existing unrelated files in the npm output directory are left untouched. If a generated file path exists as a directory, generation fails instead of deleting it.

The CLI builds the native binding in release mode. If `CARGO_TARGET_DIR` is set, the built artifact is resolved from that target directory; otherwise it is resolved from the generated bindings crate target directory.
