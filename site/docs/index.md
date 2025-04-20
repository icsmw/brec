[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.md)
[![](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml/badge.svg)](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml)
![Crates.io](https://img.shields.io/crates/v/brec)

`brec` is a tool that allows you to quickly and easily create a custom message exchange protocol with resilience to data "corruption" and the ability to extract messages from mixed streams (i.e., streams containing not only `brec` packets but also any other data). `brec` is developed for designing your own custom binary protocol — without predefined message formats or rigid schemas.

> **Notice**: Public Beta
>
> The `brec` is currently in a public beta phase. Its core functionality has demonstrated strong reliability under heavy stress testing, and the system is considered stable for most use cases.
>
> However, the public API is still evolving as we work to support a wider range of scenarios.
> We welcome your feedback and would be grateful if you share which features or improvements would make brec more valuable for your needs.
>
> **Thanks for being part of the journey!**

## Key Features

- **Protocol without constraints** – Unlike many alternatives, `brec` doesn’t enforce a fixed message layout. Instead, you define your own building blocks (`blocks`) and arbitrary payloads (`payloads`), combining them freely into custom packets.
- **Stream-recognizable messages** – Each block, payload, and packet is automatically assigned a unique signature, making them easily discoverable within any byte stream.
- **Built-in reliability** – All parts of a packet (blocks, payloads, and headers) are automatically linked with their own CRC checksums to ensure data integrity.
- **Stream-aware reading** – `brec` includes a powerful streaming reader capable of extracting packets even from noisy or corrupted streams — skipping irrelevant or damaged data without breaking.
- **Non-packet data is preserved** – When reading mixed streams, unrecognized data is not lost. You can capture and process it separately using rules and callbacks.
- **Persistent storage layer** – `brec` provides a high-performance storage engine for persisting packets. Its slot-based layout enables fast indexed access, filtering, and direct access by packet index.
- **High performance** – Parsing performance is on par with the most optimized binary parsers (see the Performance section in [performance](stability/performance.md)).
- **Simple to use** – Just annotate your structs with #[block] or #[payload], and brec takes care of the rest — your protocol is ready to go.

## Recent Performance Test Results

| Test                    | Mode      | Size    | Rows        | Time (ms) | Iterations |
|-------------------------|-----------|---------|-------------|-----------|------------|
| Storage (brec)          | Filtering | 908 Mb  | 140,000     | **612**   | 10         |
| Storage (brec)          | Reading   | 908 Mb  | 1,000,000   | 987       | 10         |
| JSON                    | Reading   | 919 Mb  | 1,000,000   | 597       | 10         |
| JSON                    | Filtering | 919 Mb  | 140,000     | 608       | 10         |
| Binary Stream (brec)    | Reading   | 831 Mb  | 1,000,000   | 764       | 10         |
| Binary Stream (brec)    | Filtering | 831 Mb  | 140,000     | **340**   | 10         |
| Plain Text              | Reading   | 774 Mb  | 1,000,000   | 247       | 10         |
| Plain Text              | Filtering | 774 Mb  | 150,000     | 276       | 10         |
| Streamed Storage (brec) | Filtering | 908 Mb  | 140,000     | **355**   | 10         |
| Streamed Storage (brec) | Reading   | 908 Mb  | 1,000,000   | 790       | 10         |

See more details in the [test](stability/tests.md) about how tests are performed and what they mean.
