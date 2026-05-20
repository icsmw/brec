# Brec C# Bindings Generator

`brec_csharp_cli` generates two artifacts from `brec.scheme.json`:

- a standalone Rust bindings crate named `bindings`;
- a generated C# project named `Protocol.csproj`.

The Rust crate depends on the source protocol crate, builds a native C ABI library, and exposes:

- `decode_block` / `encode_block`
- `decode_payload` / `encode_payload`
- `decode_packet` / `encode_packet`

The generated C# project wraps those functions with safe handles and byte helpers for block, payload, and packet roundtrips.

Typical usage:

```bash
brec_csharp_cli \
  --scheme path/to/protocol/target/brec.scheme.json \
  --protocol path/to/protocol \
  --bindings-out path/to/generated/bindings \
  --csharp-out path/to/generated/csharp
```

For a repository-local reference, see `e2e-gen/csharp/test.sh`.
