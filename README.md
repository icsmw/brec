[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.txt)
[![](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml/badge.svg)](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml)
![Coverage](.github/artifacts/coverage.svg)
[![](https://github.com/icsmw/brec/actions/workflows/coverage_report.yml/badge.svg)](https://github.com/icsmw/brec/actions/workflows/coverage_report.yml)
[![](https://github.com/icsmw/brec/actions/workflows/codeql.yml/badge.svg)](https://github.com/icsmw/brec/actions/workflows/codeql.yml)
![Crates.io](https://img.shields.io/crates/v/brec)

`brec` is a tool that allows you to quickly and easily create a custom message exchange protocol with resilience to data "corruption" and the ability to extract messages from mixed streams (i.e., streams containing not only `brec` packets but also any other data). `brec` is developed for designing your own custom binary protocol - without predefined message formats or rigid schemas.

> **Notice**: Public Beta
>
> The `brec` is currently in a public beta phase. Its core functionality has demonstrated strong reliability under heavy stress testing, and the system is considered stable for most use cases.
>
> However, the public API is still evolving as we work to support a wider range of scenarios.
> We welcome your feedback and would be grateful if you share which features or improvements would make brec more valuable for your needs.
>
> **Thanks for being part of the journey!**

## Key Features

- **Protocol without constraints** - Unlike many alternatives, `brec` doesn’t enforce a fixed message layout. Instead, you define your own building blocks (`blocks`) and arbitrary payloads (`payloads`), combining them freely into custom packets.
- **Stream-recognizable messages** - Each block, payload, and packet is automatically assigned a unique signature, making them easily discoverable within any byte stream.
- **Built-in reliability** - All parts of a packet (blocks, payloads, and headers) are automatically linked with their own CRC checksums to ensure data integrity.
- **Stream-aware reading** - `brec` includes a powerful streaming reader capable of extracting packets even from noisy or corrupted streams - skipping irrelevant or damaged data without breaking.
- **Non-packet data is preserved** - When reading mixed streams, unrecognized data is not lost. You can capture and process it separately using rules and callbacks.
- **Persistent storage layer** - `brec` provides a high-performance storage engine for persisting packets. Its slot-based layout enables fast indexed access, filtering, and direct access by packet index.
- **Optional file observer** - Enable the `observer` feature to watch storage files and consume newly appended packets asynchronously.
- **Payload runtime context** - Payloads may use explicit runtime state during encode/decode, which makes features like custom context-aware payload logic and crypto integration possible.
- **Optional payload encryption** - With the `crypt` feature, selected payloads can be encrypted while other payloads in the same protocol remain open.
- **Optional resilient compatibility** - With the `resilient` feature, older readers can skip unknown blocks and payloads during protocol evolution.
- **Optional Node.js bridge (N-API)** - With the `napi` feature, protocol objects can be converted directly between Rust and JavaScript without JSON conversion as an intermediate transport.
- **Optional WASM bridge (`wasm-bindgen`)** - With the `wasm` feature, protocol objects can be converted directly between Rust and JavaScript in browser/wasm runtimes, without JSON as an intermediate transport.
- **Optional Java bridge (JNI)** - With the `java` feature, protocol objects can be converted directly between Rust and Java runtime objects without JSON conversion as an intermediate transport.
- **Optional C# bridge (PInvoke / C ABI)** - With the `csharp` feature, protocol objects can be converted directly between Rust packet models and a stable Rust-side value ABI for .NET-facing integrations.
- **High performance** - Parsing performance is on par with the most optimized binary parsers (see the Performance section in [documentation](https://icsmw.github.io/brec/)).
- **Simple to use** - Just annotate your structs with #[block] or #[payload], and brec takes care of the rest - your protocol is ready to go.

## Core Architectural Capabilities

- A `brec` packet is a set of blocks (from 0 to 255) plus an optional payload. This effectively gives packets a built-in indexing layer at the architecture level. The restricted type set for blocks is a deliberate trade-off: it limits what can be placed into blocks, but enables low-allocation (and often allocation-free) reads and efficient pre-filtering before payload parsing (see [performance](https://icsmw.github.io/brec/stability/performance/)).
- `brec` is schema-free in packet composition, while still strongly typed in its components. The developer defines all supported block and payload types, and packets are assembled from these building elements. This allows protocol evolution not only by adding new payload shapes, but also by extending the indexing layer in packets without breaking compatibility. In practice, this compatibility story is implemented by the [`resilient` feature](https://icsmw.github.io/brec/features/resilient/): an older parser may skip unknown blocks or payloads and still read the packet when the protocol is designed for forward-compatible evolution.

In other words, `brec` combines the flexibility of schema-free protocols with the strictness of binary formats.

## Overview

The primary unit of information in `brec` is a packet (`Packet`) - a ready-to-transmit message with a unique signature (allowing it to be recognized within mixed data) and a CRC to ensure data integrity.

A packet consists of a set of blocks (`Block`) and, optionally, a payload (`Payload`).

Blocks (`Block`) are the minimal units of information in the `brec` system. A block can contain only primitives, such as numbers, boolean values, and byte slices. A block serves as a kind of packet index, allowing for quick determination of whether a packet requires full processing (i.e., parsing the `Payload`) or can be ignored.

The payload (`Payload`) is an optional part of the packet. Unlike blocks (`Block`), it has no restrictions on the type of data it can contain - it can be a `struct` or `enum` of any complexity and nesting level.

Unlike most protocols, `brec` does not require users to define a fixed set of messages but does require them to describe blocks (`Block`) and payload data (`Payload`).

Users can construct packets (messages) by combining various sets of blocks and payloads. This means `brec` does not impose a predefined list of packets (`Packet`) within the protocol but allows them to be defined dynamically. As a result, the same block and/or payload can be used across multiple packets (messages) without any restrictions.

## Recent Performance Test Results

|     Platform      |   Case    |  Bytes   |   Rows    | Time, ms | Avg, us/row | Rate, Mbit/s | CPU, ms | RSS, Kb  | PeakRSS, Kb  | Iterations |
|-------------------|-----------|----------|-----------|----------|-------------|--------------|---------|----------|--------------|------------|
| Plain text        | Writing   | 4,986 Mb | 2,500,000 |    4,183 |        1.67 |     10000.53 |   3,947 |        2 |            2 |         10 |
| Plain text        | Reading   | 4,986 Mb | 2,500,000 |   13,220 |        5.29 |      3164.11 |  13,108 |        0 |            0 |         10 |
| Plain text        | Filtering | 4,986 Mb |   308,500 |   13,264 |       43.00 |      3153.45 |  13,371 |        0 |            0 |         10 |
| JSON              | Writing   | 5,216 Mb | 2,500,000 |   14,317 |        5.73 |      3056.29 |  14,454 |        0 |            0 |         10 |
| JSON              | Reading   | 5,216 Mb | 2,500,000 |   31,784 |       12.71 |      1376.68 |  32,046 |        0 |            0 |         10 |
| JSON              | Filtering | 5,216 Mb |   308,500 |   31,819 |      103.14 |      1375.19 |  32,069 |        0 |            0 |         10 |
| Protobuf          | Writing   | 3,409 Mb | 2,500,000 |    7,112 |        2.84 |      4021.85 |   7,180 |       31 |           29 |         10 |
| Protobuf          | Reading   | 3,409 Mb | 2,500,000 |   15,634 |    **6.25** |      1829.55 |  15,756 |        0 |            0 |         10 |
| Protobuf          | Filtering | 3,409 Mb |   308,500 |   15,638 |       50.69 |      1829.08 |  15,731 |        0 |            0 |         10 |
| FlatBuffers       | Writing   | 3,878 Mb | 2,500,000 |    7,618 |        3.05 |      4270.89 |   7,691 |        0 |            0 |         10 |
| FlatBuffers       | Reading   | 3,878 Mb | 2,500,000 |    2,571 |        1.03 |     12657.02 |   2,568 |        0 |            0 |         10 |
| FlatBuffers       | Filtering | 3,878 Mb |   308,500 |    2,294 |        7.43 |     14186.19 |   2,269 |        0 |            0 |         10 |
| FlatBuffers Owned | Writing   | 3,878 Mb | 2,500,000 |    7,717 |        3.09 |      4216.48 |   7,761 |        0 |            0 |         10 |
| FlatBuffers Owned | Reading   | 3,878 Mb | 2,500,000 |    3,948 |        1.58 |      8240.88 |   3,909 |        0 |            0 |         10 |
| FlatBuffers Owned | Filtering | 3,878 Mb |   308,500 |    3,850 |       12.48 |      8452.03 |   3,868 |        0 |            0 |         10 |
| Brec Storage      | Writing   | 3,476 Mb | 2,500,000 |   11,935 |        4.77 |      2443.55 |  12,042 |   17,317 |       17,304 |         10 |
| Brec Storage      | Reading   | 3,476 Mb | 2,500,000 |   15,315 |        6.13 |      1904.24 |  14,927 |   14,952 |       15,343 |         10 |
| Brec Storage      | Filtering | 3,476 Mb |   308,500 |    5,394 |       17.48 |      5406.84 |   4,858 |   15,033 |       15,424 |         10 |
| Brec Stream       | Writing   | 3,457 Mb | 2,500,000 |   10,233 |        4.09 |      2834.28 |  10,331 |        0 |            0 |         10 |
| Brec Stream       | Reading   | 3,457 Mb | 2,500,000 |   14,529 |    **5.81** |      1996.19 |  14,632 |        4 |            4 |         10 |
| Brec Stream       | Filtering | 3,457 Mb |   308,500 |    4,231 |       13.72 |      6854.39 |   4,234 |        4 |            4 |         10 |

See more details in the [documentation](https://icsmw.github.io/brec/) about how tests are performed and what they mean.

## Documentation

The main documentation for this crate lives in [documentation](https://icsmw.github.io/brec/).

Useful entry points:

- [Integrations Overview](https://icsmw.github.io/brec/integrations/)
- [C# (Rust <-> C#)](https://icsmw.github.io/brec/integrations/csharp/)
- [Getting Started](https://icsmw.github.io/brec/getting_started/)
- [Payloads](https://icsmw.github.io/brec/parts/payloads/)
- [Payload Context](https://icsmw.github.io/brec/parts/context/)
- [Crypt](https://icsmw.github.io/brec/features/crypt/)
- [Resilient Compatibility](https://icsmw.github.io/brec/features/resilient/)
- [NAPI (Rust <-> JS)](https://icsmw.github.io/brec/integrations/napi/)
- [WASM (Rust <-> JS)](https://icsmw.github.io/brec/integrations/wasm/)
- [Java (Rust <-> Java)](https://icsmw.github.io/brec/integrations/java/)

## Workspace Layout

The repository is organized by responsibility:

- `lib/core` - the public `brec` crate and the protocol/runtime API
- `lib/consts` - shared wire-format constants
- `generator/*` - parsing and proc-macro code generation
- `integration/*` - language-specific runtime bridges and generators
- `tests/*` - CI-oriented test suites covering the core functionality; the same areas also have stress runs where the total generated data volume can reach roughly 40 GB
- `scripts/*` - helper scripts for coverage collection, reporting, and other repository maintenance tasks
- `examples/*` - real usage examples for common `brec` scenarios
- `e2e/*` - end-to-end examples of real integration with other languages and runtimes such as Node.js, WASM, Java, and C#
- `site/*` - project documentation source
- `measurements/*` - performance evaluation and comparison against major alternatives

This keeps the public crate small while allowing integration-specific logic to evolve independently.

## Contributing

We welcome contributions of all kinds - bug reports, performance improvements, documentation fixes, or new features.

[Click here to view it](CONTRIBUTING.md)
