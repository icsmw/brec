[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.txt)
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
- **High performance** – Parsing performance is on par with the most optimized binary parsers (see the Performance section in [documentation](https://icsmw.github.io/brec/)).
- **Simple to use** – Just annotate your structs with #[block] or #[payload], and brec takes care of the rest — your protocol is ready to go.

## Overview

The primary unit of information in `brec` is a packet (`Packet`) — a ready-to-transmit message with a unique signature (allowing it to be recognized within mixed data) and a CRC to ensure data integrity.

A packet consists of a set of blocks (`Block`) and, optionally, a payload (`Payload`).

Blocks (`Block`) are the minimal units of information in the `brec` system. A block can contain only primitives, such as numbers, boolean values, and byte slices. A block serves as a kind of packet index, allowing for quick determination of whether a packet requires full processing (i.e., parsing the `Payload`) or can be ignored.

The payload (`Payload`) is an optional part of the packet. Unlike blocks (`Block`), it has no restrictions on the type of data it can contain—it can be a `struct` or `enum` of any complexity and nesting level.

Unlike most protocols, `brec` does not require users to define a fixed set of messages but does require them to describe blocks (`Block`) and payload data (`Payload`).

Users can construct packets (messages) by combining various sets of blocks and payloads. This means `brec` does not impose a predefined list of packets (`Packet`) within the protocol but allows them to be defined dynamically. As a result, the same block and/or payload can be used across multiple packets (messages) without any restrictions.

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

See more details in the [documentation](https://icsmw.github.io/brec/) about how tests are performed and what they mean.

## Documentation

The main documentation for this crate lives in [documentation](https://icsmw.github.io/brec/).

## Contributing

We welcome contributions of all kinds — bug reports, performance improvements, documentation fixes, or new features.

[Click here to view it](CONTRIBUTING.md)