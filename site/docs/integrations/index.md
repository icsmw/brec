# Integrations

`brec` integrations are built around one core rule: packet parsing, validation, CRC handling, stream reading, and payload codecs stay in Rust. Other runtimes receive bindings to those capabilities, not a reimplementation of the protocol engine.

This is an intentional architectural choice.

In many ecosystems, such as protobuf-style toolchains, the generated code on the target platform becomes the place where encoding and decoding logic actually runs. `brec` takes a different route. We keep the binary protocol logic in Rust permanently and export language-facing adapters that convert runtime objects to and from the Rust packet model.

For users, this means:

- the wire format is defined once and executed by one implementation
- packet validation and codec behavior stay consistent across runtimes
- integration code does not need to duplicate packet parsing rules in JavaScript, Java, or another target platform

For developers, this means the integration layer is primarily about object conversion and API ergonomics, not about cloning the protocol engine for every language.

## Architectural Split

After the refactoring, `brec` is intentionally split into two layers.

### `lib/core`: the thin public entry layer

The `brec` crate in `lib/core` keeps the stable protocol model and the user-facing API:

- packet, block, payload, storage, crypt, and reader logic
- common traits such as `BlockDef`, `PayloadDef`, `PayloadEncode`, `PayloadDecode`, and packet readers/writers
- the main macros users invoke from their protocol crate
- feature-gated reexports of integration derives and helper crates
- thin wrapper methods such as `Packet::decode_wasm`, `Packet::encode_napi`, and `Packet::decode_java`

This layer should stay small. It defines the protocol engine and exposes integration entry points, but it does not contain the full language-specific implementation.

### `integration/<lang>`: language-specific logic

Each integration keeps its own implementation in dedicated crates under `integration/`:

- `integration/<lang>/lib` contains the runtime bridge for that platform
- `integration/<lang>/gen` contains code generation used by derives and generated protocol glue
- `integration/<lang>/macro` contains language-specific derive macros when that integration needs them

Examples from the current workspace:

- `integration/node/{lib,gen,macro}` for N-API / Node.js
- `integration/wasm/{lib,gen,macro}` for `wasm-bindgen`
- `integration/java/{lib,gen,macro}` for JNI / Java
- `integration/csharp/{lib,gen}` for the C# bridge

Not every integration needs exactly the same crate set. For example, C# is currently wired through dedicated `lib` and `gen` crates, while the pages in this section focus on the integrations that already have end-user documentation.

In practice, `lib/core` enables these crates through feature flags and reexports only what protocol authors need at the call site.

## What Actually Crosses The Boundary

The integration boundary is not the packet codec itself. The boundary is the object model exposed to the target runtime.

Typical flow:

1. Rust reads packet bytes using the normal `brec` packet reader.
2. Rust decodes blocks and payload using the same Rust-side traits and codecs as everywhere else.
3. The integration crate converts the resulting Rust representation into a platform object:
   - `JsValue` for wasm
   - N-API values for Node.js
   - JNI objects for Java
4. For encoding, the process runs in reverse: the integration crate maps platform objects back into Rust values, then `brec` writes packet bytes using the same Rust implementation.

That distinction matters. `brec` does not "port the codec" into JavaScript or Java. It ports a binding surface that lets those runtimes work with protocol-shaped values while Rust remains the execution engine for packet semantics.

## Why This Matters

This separation gives several practical benefits:

- one source of truth for binary behavior
- no per-language drift in CRC rules, packet layout handling, or payload codec behavior
- easier testing because protocol correctness remains concentrated in Rust
- lower maintenance cost when integrations evolve independently
- smaller `lib/core`, with fewer platform-specific concerns mixed into the crate

It also sets expectations correctly: language integrations are not independent protocol stacks. They are adapters around the Rust core.

## How To Read The Rest Of This Section

Use the language-specific pages when you need concrete integration details:

- [C#](csharp.md)
- [NAPI](napi.md)
- [WASM](wasm.md)
- [Java](java.md)

Those pages explain runtime-specific object shapes, feature flags, derives, and binding entry points. This page explains the shared model behind all of them.
