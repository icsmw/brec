## 0.2.0 (28.06.2025)

### Added

- Feature `locked_storage`: introduces `FileStorage`, a file-backed wrapper around `StorageDef` with advisory locking via `.lock` files to coordinate exclusive access across processes. Enabled through the `fs4` crate and supports configurable timeout and retry interval.

## 0.1.4 (21.06.2025)

### Changes

Full compatibility with tokio::spawn and other `Send + 'static` async environments:

- All internal callback types (`RuleFnDef`) now explicitly implement `Send + 'static`.
- `StorageDef` and all related structures are now `Send`, making them safe to use inside async tasks and multi-threaded executors.

This enables direct integration of `brec` into tokio-based systems without additional wrappers.

This change does not make `brec` asynchronous — I/O operations remain blocking (`std::fs`). However, it is now safe to use `StorageDef` in asynchronous environments with care (e.g., inside `tokio::task::block_in_place` or `spawn_blocking`).

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