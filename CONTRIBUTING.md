# Contributing to `brec`

First of all, thank you for your interest in contributing to `brec`!  
We welcome contributions of all kinds — bug reports, performance improvements, documentation fixes, or new features.

Please follow the guidelines below to ensure a smooth review and merge process.

---

## Workflow

The contribution process follows the standard GitHub workflow:

1. **Fork** the repository.
2. **Create a new branch** for your feature or fix.
3. **Implement your changes.**
4. **Open a Pull Request**, describing:
   - what the changes do,
   - why they are necessary or useful.

Please be clear and thorough in your description — it helps maintainers understand your intentions quickly.

---

## Performance Considerations (Mandatory)

`brec` is designed with high performance and low overhead in mind.  
If your changes affect core logic (reading, writing, parsing, or streaming), you must ensure there is **no performance degradation**.

To measure this, run the existing benchmark test before and after your changes:

```bash
cd tests/measurements
cargo test --release -- --nocapture
```

1. Run this test **on the unmodified main branch**, and note the results.
2. Then run the same test **after your changes**.

Include the performance comparison in your pull request description.  
Unjustified performance regressions may result in the PR being rejected — unless the change is required to fix a critical bug.

---

## Testing Requirements

There are two levels of tests:

### 1. `test.sh` — Fast CI Tests

A lightweight test script that runs quickly and is used in the CI pipeline.

You can run it locally:

```bash
./test.sh
```

> This test is **not sufficient** for merging critical changes.

---

### 2. `stress.sh` — Mandatory for Critical Paths

The full stress test script generates over **40 GB of test data** to validate robustness.

Run it manually before submitting PRs that:
- modify file I/O logic,
- affect reading/writing of packets,
- or change serialization/deserialization behavior.

```bash
./stress.sh
```

This is the **final gate** for acceptance of changes.

---

## Linting & Style

Before submitting a pull request, run:

```bash
./lint.sh
```

This checks formatting, lints the code, and ensures consistency.

---

## Documentation Guidelines

- All **public methods** must be documented with clear descriptions of parameters and return values.
- **Private methods** may remain undocumented, although **documentation is encouraged** wherever it helps future maintainers.

---

Thank you again for contributing — we appreciate your time and effort!  
If you have any questions, feel free to open an issue or discussion thread.
