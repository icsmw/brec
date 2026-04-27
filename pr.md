## Summary

This PR completes the large workspace refactor around integrations and adds first-class C# support.

The main change is architectural: language-specific generation and runtime logic are no longer embedded directly in the main crates. Instead, the workspace is split into focused `lib`, `generator`, and `integration` crates, while the public `brec` crate remains the user-facing entry point.

On top of that refactor, the integration surface now includes C# alongside the existing Node.js, WASM, and Java support, with matching end-to-end coverage and documentation updates.

## What Changed

- split the workspace into dedicated `lib/*`, `generator/*`, and `integration/*` crates
- moved integration-specific codegen and runtime code out of the monolithic crates into per-language crates
- added C# integration support and end-to-end coverage
- updated CI, coverage, scripts, and repository documentation to the new layout

## Notes

This PR is intentionally infrastructure-heavy. Most of the value is in clearer workspace boundaries, easier maintenance of per-language integrations, and a thinner public `brec` layer for protocol authors.
