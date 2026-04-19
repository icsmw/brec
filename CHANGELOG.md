## Unreleased

### Added

- Rust feature flag `napi`: direct Rust <-> JavaScript object conversion for generated `Block`, `Payload`, and `PacketDef` protocol types.
- Rust feature flag `wasm`: direct Rust <-> JavaScript object conversion for generated `Block`, `Payload`, and `PacketDef` protocol types in `wasm-bindgen` runtimes.
- Rust feature flag `java`: direct Rust <-> Java object conversion for generated `Block`, `Payload`, and `PacketDef` protocol types in JNI-based runtimes.
- Dedicated documentation section for N-API usage, JS reflection model, and nested payload requirements.
- Dedicated documentation section for WASM usage, JS reflection model, and nested payload requirements.
- Dedicated documentation section for Java usage, object reflection model, and nested payload requirements.

## 0.3.0 (20.03.2026)

### Added

- Rust feature flag `observer`: observer module, including stream-based observing support.
- Rust feature flag `crypt`: encryption/decryption support in read/write pipeline with related options and policies.
- Rust feature flag `resilient`: resilient reading path with dedicated stress/integration coverage.
- Functionality: seek support for iterating and reading from the N-th packet.
- Functionality: support of custom options across core APIs.
- New examples for context, packets, crypt, bincode and reorganized wasm example layout.

### Changes

- Storage internals were split into dedicated reader/writer modules.
- Updated observe, filters, and `BlockReferred` APIs.
- Added packet-level `PacketReadStatus`.
- Extended `SignatureDismatch` diagnostics in core and macros.
- Reading path optimization to avoid unnecessary payload reads.
- Project updated to Rust 2024 edition.
- Expanded documentation and performance/measurement reports (including CPU/RSS metrics and heavy dataset scenarios).

### Fixes

- Corrected length-check logic in packet processing.
- Fixed macro test generator behavior for stress scenarios.
- Various test/workflow/documentation consistency fixes across workspace crates.

## 0.2.0 (28.06.2025)

### Added

- Feature `locked_storage`: introduces `FileStorage`, a file-backed wrapper around `StorageDef` with advisory locking via `.lock` files to coordinate exclusive access across processes. Enabled through the `fs4` crate and supports configurable timeout and retry interval.

## 0.1.4 (21.06.2025)

### Changes

Full compatibility with tokio::spawn and other `Send + 'static` async environments:

- All internal callback types (`RuleFnDef`) now explicitly implement `Send + 'static`.
- `StorageDef` and all related structures are now `Send`, making them safe to use inside async tasks and multi-threaded executors.

This enables direct integration of `brec` into tokio-based systems without additional wrappers.

This change does not make `brec` asynchronous - I/O operations remain blocking (`std::fs`). However, it is now safe to use `StorageDef` in asynchronous environments with care (e.g., inside `tokio::task::block_in_place` or `spawn_blocking`).

## 0.1.3 (28.03.2025)

### Fixes

- Fixed issue with `count()` on `StorageDef`
- Fixed missed setup (on load) of slot's locator 

### Changes

- Added tests for `count()` on `StorageDef`
- Updated documentation

## 0.1.2 (27.03.2025)

### Fixes

- Consider full path of derives on blocks and payloads parsing

### Changes

- Update documentation
- Add wasm example
- Include examples build into CI pipeline

## 0.1.1 (24.03.2025)

### Changes

- Update documentation
