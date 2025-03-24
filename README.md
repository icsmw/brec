`brec` is a tool that allows you to quickly and easily create a custom message exchange protocol with resilience to data "corruption" and the ability to extract messages from mixed streams (i.e., streams containing not only `brec` packets but also any other data).

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

See more details in the [documentation](./brec/README.md) about how tests are performed and what they mean.

## Documentation

The main documentation for this crate lives in [`brec/README.md`](./brec/README.md).

[Click here to view it](brec/README.md)

## Contributing

We welcome contributions of all kinds — bug reports, performance improvements, documentation fixes, or new features.

[Click here to view it](CONTRIBUTING.md)