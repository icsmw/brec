[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.md)
[![](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml/badge.svg)](https://github.com/icsmw/brec/actions/workflows/on_pull_request.yml)
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
- **High performance** - Parsing performance is on par with the most optimized binary parsers (see the Performance section in [performance](stability/performance.md)).
- **Simple to use** - Just annotate your structs with #[block] or #[payload], and brec takes care of the rest - your protocol is ready to go.

## Core Architectural Capabilities

- A `brec` packet is a set of blocks (from 0 to 255) plus an optional payload. This effectively gives packets a built-in indexing layer at the architecture level. The restricted type set for blocks is a deliberate trade-off: it limits what can be placed into blocks, but enables low-allocation (and often allocation-free) reads and efficient pre-filtering before payload parsing (see [performance](https://icsmw.github.io/brec/stability/performance/)).
- `brec` is schema-free in packet composition, while still strongly typed in its components. The developer defines all supported block and payload types, and packets are assembled from these building elements. This allows protocol evolution not only by adding new payload shapes, but also by extending the indexing layer in packets without breaking compatibility. In practice, this compatibility story is implemented by the [`resilient` feature](https://icsmw.github.io/brec/stability/resilient/): an older parser may skip unknown blocks or payloads and still read the packet when the protocol is designed for forward-compatible evolution.

In other words, `brec` combines the flexibility of schema-free protocols with the strictness of binary formats.

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

See more details in the [test](stability/tests.md) about how tests are performed and what they mean.
