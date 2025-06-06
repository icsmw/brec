`brec` is a tool that allows you to quickly and easily create a custom message exchange protocol with resilience to data "corruption" and the ability to extract messages from mixed streams (i.e., streams containing not only `brec` packets but also any other data). `brec` is developed for designing your own custom binary protocol — without predefined message formats or rigid schemas.

> **Notice**: Public Beta
>
> The `brec` is currently in a public beta phase. Its core functionality has demonstrated strong reliability under heavy stress testing, and the system is considered stable for most use cases.
>
> However, the public API is still evolving as we work to support a wider range of scenarios.
> We welcome your feedback and would be grateful if you share which features or improvements would make brec more valuable for your needs.
>
> **Thanks for being part of the journey!**

# Key Features

- **Protocol without constraints** – Unlike many alternatives, `brec` doesn’t enforce a fixed message layout. Instead, you define your own building blocks (`blocks`) and arbitrary payloads (`payloads`), combining them freely into custom packets.
- **Stream-recognizable messages** – Each block, payload, and packet is automatically assigned a unique signature, making them easily discoverable within any byte stream.
- **Built-in reliability** – All parts of a packet (blocks, payloads, and headers) are automatically linked with their own CRC checksums to ensure data integrity.
- **Stream-aware reading** – `brec` includes a powerful streaming reader capable of extracting packets even from noisy or corrupted streams — skipping irrelevant or damaged data without breaking.
- **Non-packet data is preserved** – When reading mixed streams, unrecognized data is not lost. You can capture and process it separately using rules and callbacks.
- **Persistent storage layer** – `brec` provides a high-performance storage engine for persisting packets. Its slot-based layout enables fast indexed access, filtering, and direct access by packet index.
- **High performance** – Parsing performance is on par with the most optimized binary parsers (see the Performance section).
- **Simple to use** – Just annotate your structs with #[block] or #[payload], and brec takes care of the rest — your protocol is ready to go.

# General Overview

The primary unit of information in `brec` is a packet (`Packet`) — a ready-to-transmit message with a unique signature (allowing it to be recognized within mixed data) and a CRC to ensure data integrity.

A packet consists of a set of blocks (`Block`) and, optionally, a payload (`Payload`).

![Scheme](./assets/scheme.svg)

Blocks (`Block`) are the minimal units of information in the `brec` system. A block can contain only primitives, such as numbers, boolean values, and byte slices. A block serves as a kind of packet index, allowing for quick determination of whether a packet requires full processing (i.e., parsing the `Payload`) or can be ignored.

The payload (`Payload`) is an optional part of the packet. Unlike blocks (`Block`), it has no restrictions on the type of data it can contain—it can be a `struct` or `enum` of any complexity and nesting level.

Unlike most protocols, `brec` does not require users to define a fixed set of messages but does require them to describe blocks (`Block`) and payload data (`Payload`).

Users can construct packets (messages) by combining various sets of blocks and payloads. This means `brec` does not impose a predefined list of packets (`Packet`) within the protocol but allows them to be defined dynamically. As a result, the same block and/or payload can be used across multiple packets (messages) without any restrictions.

# Features

## Simplicity of Protocol Type Definition
`brec` includes powerful macros that allow defining the components of a protocol with minimal effort. For example, to define a structure as a block (`Block`), you simply need to use the `block` macro:

```ignore
#[brec::block]
pub struct MyBlock {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_i8: i8,
    pub field_i16: i16,
    pub field_i32: i32,
    pub field_i64: i64,
    pub field_i128: i128,
    pub field_f32: f32,
    pub field_f64: f64,
    pub field_bool: bool,
    pub blob_a: [u8; 1],
    pub blob_b: [u8; 100],
    pub blob_c: [u8; 1000],
    pub blob_d: [u8; 10000],
}
```

The `block` macro automatically generates all the necessary code for `MyBlock` to function as a block. Specifically, it adds:
- A unique block signature based on its name and path.
- A `CRC` field to ensure data integrity.

All user-defined blocks are ultimately included in a generated enumeration `Block`, as shown in the example:

```ignore
#[brec::block]
pub struct MyBlockA {
    pub field: u8,
    pub blob: [u8; 100],
}

#[brec::block]
pub struct MyBlockB {
    pub field_u16: u16,
    pub field_u32: u32,
}

#[brec::block]
pub struct MyBlockC {
    pub field: u64,
    pub blob: [u8; 10000],
}

// Instruct `brec` to generate and include all protocol types
brec::generate!();

// Generated by `brec`
pub enum Block {
    MyBlockA(MyBlockA),
    MyBlockB(MyBlockB),
    MyBlockC(MyBlockC),
}
```

The generated `Block` enumeration is always returned to the user as a result of message (packet) parsing, allowing for easy identification of blocks by their names.

Similarly to blocks, defining a payload can be done with a simple call to the `payload` macro.

```ignore
#[derive(serde::Deserialize, serde::Serialize)]
pub enum MyNestedEntity {
    One(String),
    Two(Vec<u8>),
    Three,
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayload {
    pub field_u8: u8,
    pub field_u16: u16,
    pub field_u32: u32,
    pub field_u64: u64,
    pub field_u128: u128,
    pub field_nested: MyNestedEntity,
}
```

A payload is an unrestricted set of data (unlike `Block`, which is limited to primitive data). It can be either a `struct` or an `enum` with unlimited nesting levels. Additionally, there are no restrictions on the use of generic types.

`brec` imposes only one requirement for payloads: they must implement the traits `PayloadEncode` and `PayloadDecode<T>`, which enable encoding and decoding of data into target types.

Out of the box (with the `bincode` feature), `brec` provides automatic support for the required traits by leveraging the `bincode` crate. This means that to define a fully functional payload, it is enough to use `#[payload(bincode)]`, eliminating the need for manually implementing the `PayloadEncode` and `PayloadDecode<T>` traits. 

Note that `bincode` requires serialization and deserialization support, which is why the previous examples include `#[derive(serde::Deserialize, serde::Serialize)]`.

With `brec`, defining protocol types (blocks and payloads) is reduced to simply defining structures and annotating them with the `block` and `payload` macros.

## Simple Packet Construction
Once the protocol data types have been defined, the next step is to include the "unifying" code generated by `brec` using `brec::generate!();`

```ignore
#[brec::block]
pub struct MyBlockA { ... }

#[brec::block]
pub struct MyBlockB { ... }

#[brec::block]
pub struct MyBlockC { ... }

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadA { ... }

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadB { ... }

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadC { ... }

// Instruct `brec` to generate and include all protocol types
brec::generate!();

// Now available:

// Generalized block representation
pub enum Block {
    MyBlockA(MyBlockA),
    MyBlockB(MyBlockB),
    MyBlockC(MyBlockC),
}

// Generalized payload representation
pub enum Payload {
    MyPayloadA(MyPayloadA),
    MyPayloadB(MyPayloadB),
    MyPayloadC(MyPayloadC),
}

// Packet type
pub type Packet = brec::PacketDef<Block, Payload, Payload>;
```

Once all protocol types are defined and the unifying code is generated, you can start creating packets:

```ignore
let my_packet = Packet::new(
    // You are limited to 255 blocks per packet.
    vec![
        Block::MyBlockA(MyBlockA::default()),
        Block::MyBlockC(MyBlockC::default()),
    ],
    // Note: payload is optional
    Some(Payload::MyPayloadA(MyPayloadA::default()))
);
```

At this point, your protocol is ready for use. `Packet` implements all the necessary methods for reading from and writing to a data source.

## Performance, Security, and Efficiency
`brec` is a binary protocol, meaning data is always transmitted and stored in a binary format.

The protocol ensures security through the following mechanisms:
- Each block includes a unique signature generated based on the block's name. Name conflicts within a single crate are eliminated, as the module path is taken into account.
- Similar to blocks, each payload also has a unique signature derived from its name.
- Additionally, `Packet` itself has a fixed 64-bit signature.

These features enable reliable entity recognition within a data stream. Furthermore, blocks, payloads, and the packet itself have their own CRCs. While blocks always use a 32-bit CRC, payloads allow for optional support of 64-bit or 128-bit CRC to enhance protocol security.

`brec` ensures maximum performance through the following optimizations:
- Minimization of data copying and cloning operations.
- Incremental packet parsing: first, blocks are parsed, allowing the user to inspect them and decide whether the packet should be fully parsed (including the payload) or skipped. This enables efficient packet filtering based on block values, avoiding the overhead of parsing a heavy payload.
- If data integrity verification is not required, `brec` allows CRC to be disabled for all types or selectively. This improves performance by eliminating the need for hash calculations.

The conceptual separation of a packet into blocks and a payload allows users to efficiently manage traffic load. Blocks can carry "fast" information that requires quick access, while the payload can implement more complex encoding/decoding mechanisms (such as data compression). Filtering based on blocks helps avoid unnecessary operations on the payload when they are not required.

# Protocol Definition

## Blocks

Any structure can be used as a block, provided it contains fields of the following types:

| Type   |
|--------|
| u8     |
| u16    |
| u32    |
| u64    |
| u128   |
| i8     |
| i16    |
| i32    |
| i64    |
| i128   |
| f32    |
| f64    |
| bool   |
| [u8; n] |

A block can also include an `enum`, but only if it can be converted into one of the supported types. For example:

```ignore
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

// Ensure conversion of `Level` to `u8`
impl From<&Level> for u8 {
    fn from(value: &Level) -> Self {
        match value {
            Level::Err => 0,
            Level::Warn => 1,
            Level::Debug => 2,
            Level::Info => 3,
        }
    }
}

// Ensure conversion of `u8` to `Level`
impl TryFrom<u8> for Level {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Level::Err),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Debug),
            3 => Ok(Level::Info),
            invalid => Err(format!("{invalid} isn't a valid value for Level")),
        }
    }
}

#[block]
pub struct BlockWithEnum {
    pub level: Level,
    pub data: [u8; 200],
}
```

A structure is declared as a block using the `block` macro. If the block is located outside the visibility scope of the `brec::generate!()` code generator call, the module path must be specified using the `path` directive:

```ignore
#[block(path = mod_a::mod_b)]
pub struct BlockWithEnum {
    pub level: Level,
    pub data: [u8; 200],
}

// Since the `path` directive is the default, the path can also be defined directly:

#[block(mod_a::mod_b)]
pub struct BlockWithEnum {
    pub level: Level,
    pub data: [u8; 200],
}
```

However, in most cases, this approach is not recommended. It is better to ensure the visibility of all blocks at the location where the `brec::generate!()` code generator is invoked.

The code generator provides support for the following traits:

| Trait                  | Available Methods | Return Type | Purpose |
|------------------------|------------------|-------------|---------|
| `ReadBlockFrom`       | `read<T: std::io::Read>(buf: &mut T, skip_sig: bool)` | `Result<Self, Error>` | Attempts to read a block with an option to skip signature recognition (e.g., if the signature has already been read and the user knows exactly which block is being parsed). Returns an error if reading fails for any reason. |
| `ReadBlockFromSlice`  | `read_from_slice<'b>(src: &'b [u8], skip_sig: bool)` | `Result<Self, Error>` | Attempts to read a block from a byte slice, with an option to skip signature verification. Unlike other methods, this returns a reference to a block representation generated by `brec`, rather than the user-defined block itself (see details below). |
| `TryReadFrom`         | `try_read<T: std::io::Read + std::io::Seek>(buf: &mut T)` | `Result<ReadStatus<Self>, Error>` | Attempts to read a block, but instead of returning an error when data is insufficient, it returns a corresponding read status. Also, it moves the source's position only upon successful reading, otherwise keeping it unchanged. |
| `TryReadFromBuffered` | `try_read<T: std::io::BufRead>(reader: &mut T)` | `Result<ReadStatus<Self>, Error>` | Identical to `TryReadFrom`, except it returns a reference to the generated block representation (see details below). |
| `WriteTo`             | `write<T: std::io::Write>(&self, writer: &mut T)` | `std::io::Result<usize>` | Equivalent to the standard `write` method, returning the number of bytes written to the output. Does not guarantee flushing to the output, so calling `flush` is required if such guarantees are needed. |
| `WriteTo`             | `write_all<T: std::io::Write>(&self, writer: &mut T)` | `std::io::Result<()>` | Equivalent to the standard `write_all` method. |
| `WriteVectoredTo`     | `slices(&self)` | `std::io::Result<brec::IoSlices>` | Returns the binary representation of the block as slices. |
| `WriteVectoredTo`     | `write_vectored<T: std::io::Write>(&self, buf: &mut T)` | `std::io::Result<usize>` | Attempts a vectored write of the block (analogous to the standard `write_vectored`). |
| `WriteVectoredTo`     | `write_vectored_all<T: std::io::Write>(&self, buf: &mut T)` | `std::io::Result<()>` | Attempts a vectored write of the block (analogous to the standard `write_vectored_all`). |
| `CrcU32`             | `fn crc(&self)` | `[u8; 4]` | Computes the CRC of the block. |
| `StaticSize`         | `fn ssize()` | `u64` | Returns the size of the block in bytes. |

It is evident that all the listed write methods transmit the binary representation of the block to the output.

As mentioned earlier, `brec` also generates a reference representation of a block:

```ignore
#[block]
pub struct BlockWithEnum {
    pub level: Level,
    pub data: [u8; 200],
}

// Generated by `brec`
struct BlockWithEnumReferred<'a> {
    pub level: Level,
    pub data: &'a [u8; 200],
}
```

As seen above, the reference representation of a block does not store the slice itself but rather a reference to it. This allows the user to inspect the block while avoiding unnecessary data copying. If needed, the referenced block can be easily converted into the actual block using `.into()`, at which point the data will be copied from the source.

### Block Parameters

The `block` macro can be used with the following directives:

- `path = mod::mod` – Specifies the module path for the block if it is not directly imported at the location of `brec::generate!()`. This approach is not recommended (it is better to ensure block visibility at the generator call site), but it is not inherently inefficient or unstable. However, using this method may make future code maintenance more difficult.
- `no_crc` – Disables CRC verification for the block. Note that this does not remove the CRC field from the binary representation of the block. The CRC field will still be present but filled with zeros, and no CRC calculation will be performed.

## Payloads

`brec` does not impose any restrictions on the type of data that can be defined as a payload. However, a payload must implement the following traits:

### Required Traits

| Trait                  | Method | Return Type | Description |
|------------------------|--------|-------------|-------------|
| `PayloadSize`         | `size(&self)` | `std::io::Result<u64>` | Returns the size of the payload body in bytes (excluding the header). |
| `PayloadSignature`    | `sig(&self)` | `ByteBlock` | Returns the signature, which can have a variable length of 4, 8, 16, 32, 64, or 128 bytes. |
| `StaticPayloadSignature` | `ssig()` | `ByteBlock` | Similar to `sig`, but can be called without creating a payload instance. |
| `PayloadEncode`       | `encode(&self)` | `std::io::Result<Vec<u8>>` | Creates a binary representation of the payload (excluding the header). |
| `PayloadEncodeReferred` | `encode(&self)` | `std::io::Result<Option<&[u8]>>` | Creates a reference representation of the payload, if possible. Used for calculation of payload CRC and can boost performance |
| `PayloadDecode<T>`    | `decode(buf: &[u8])` | `std::io::Result<T>` | Attempts to reconstruct the payload from a byte slice. |

### Automatically Implemented Traits

The following traits are automatically applied and do not require manual implementation, though they can be overridden if needed:

| Trait                  | Method | Return Type | Description |
|------------------------|--------|-------------|-------------|
| `PayloadCrc`          | `crc(&self)` | `std::io::Result<ByteBlock>` | Returns the CRC of the payload, which can have a variable length of 4, 8, 16, 32, 64, or 128 bytes. |
| `ReadPayloadFrom`     | `read<B: std::io::Read>(buf: &mut B, header: &PayloadHeader)` | `Result<T, Error>` | Reads the payload from a source. |
| `TryReadPayloadFrom`  | `try_read<B: std::io::Read + std::io::Seek>(buf: &mut B, header: &PayloadHeader)` | `Result<ReadStatus<T>, Error>` | Attempts to read the payload from a source. If there is insufficient data, it returns a corresponding read status instead of an error. |
| `TryReadPayloadFromBuffered` | `try_read<B: std::io::BufRead>(buf: &mut B, header: &PayloadHeader)` | `Result<ReadStatus<T>, Error>` | Similar to `TryReadPayloadFrom`, but returns a reference representation of the payload (if supported) instead of the actual payload. |
| `WritePayloadWithHeaderTo` | `write<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<usize>` | Equivalent to the standard `write` method, returning the number of bytes written. Does not guarantee that data is flushed to the output, so calling `flush` is required if such guarantees are needed. |
| `WritePayloadWithHeaderTo` | `write_all<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<()>` | Equivalent to the standard `write_all` method. |
| `WriteVectoredPayloadWithHeaderTo` | `write_vectored<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<usize>` | Attempts a vectored write of the payload (analogous to the standard `write_vectored`). |
| `WriteVectoredPayloadWithHeaderTo` | `slices(&mut self)` | `std::io::Result<IoSlices>` | Returns the binary representation of the payload as slices. |
| `WriteVectoredPayloadWithHeaderTo` | `write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<()>` | Attempts a vectored write of the payload (analogous to the standard `write_vectored_all`). |

### Payload Header (`PayloadHeader`)

The payload header is not a generated structure but a static one included in the `brec` crate. `PayloadHeader` is used when writing a payload into a packet (`Packet`) and is always written before the payload body itself. When manually writing a payload to a data source, it is strongly recommended to prepend it with `PayloadHeader` to facilitate further reading from the source.

`PayloadHeader` implements the following traits:

| Trait               | Method | Return Type | Description |
|---------------------|--------|-------------|-------------|
| `ReadFrom`        | `read<T: std::io::Read>(buf: &mut T)` | `Result<Self, Error>` | Attempts to read `PayloadHeader` from a source. |
| `TryReadFrom`     | `try_read<T: std::io::Read + std::io::Seek>(buf: &mut T)` | `Result<ReadStatus<Self>, Error>` | Attempts to read `PayloadHeader`, but if data is insufficient, returns a corresponding read status instead of an error. Also, moves the source's position only on successful reading; otherwise, it remains unchanged. |
| `TryReadFromBuffered` | `try_read<T: std::io::BufRead>(reader: &mut T)` | `Result<ReadStatus<Self>, Error>` | Identical to `TryReadFrom`. |

Thus, if you are manually implementing payload reading from a source, you should first read `PayloadHeader`, and then, using the obtained header, proceed to read the payload itself.

`PayloadHeader` does not implement any traits for writing to a source. However, it provides the `as_vec()` method, which returns its binary representation.

### Automatically Supported Payload Types

Out of the box, `brec` supports `String` and `Vec<u8>` as payload types. After code generation, these will be included in the corresponding enumeration:

```ignore
brec::generate!();

pub enum Payload {
    // ..
    // User-defined payloads
    // ..
    // Default payloads
    Bytes(Vec<u8>),
    String(String),
}
```

### Defining Payloads with `bincode`

Enabling the `bincode` feature provides the simplest and most flexible way to define payload types. By specifying `#[payload(bincode)]`, any type that supports `serde` serialization and deserialization can be used as a `payload`.

```ignore
#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadStruct {
    // User-defined fields
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum MyPayloadEnum {
    // User-defined variants
}
```

### Partial Restrictions on Payload Types

It is important to note that the CRC for a payload is generated twice—once when the payload is converted into bytes and again after extraction (to compare with the CRC stored in the payload header). This imposes certain limitations on CRC verification, as `brec` does not restrict the types of data used in a payload. If a payload contains data types that do not guarantee a strict byte sequence, CRC verification will always fail due to variations in byte order. As a result, extracting such a payload from the stream will become impossible.

A simple example of this issue is `HashMap`, which does not guarantee a consistent field order upon reconstruction. For instance:

```ignore
#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadB {
    items: HashMap<String, String>,
}
```

Extracting such a payload will be impossible because the CRC will always be different (except when the number of keys in the map is ≤ 1). This issue can be resolved in several ways:

- The simplest approach is to avoid using "unstable" data types and instead choose a type that guarantees a fixed byte sequence.
- Disable CRC verification for this specific payload using the `no_crc` directive: `#[payload(no_crc)]`.
- Disable automatic CRC calculation and implement the `PayloadCrc` trait manually for the specific payload. This can be done using the `no_auto_crc` directive: `#[payload(bincode, no_auto_crc)]`.

### Payload Parameters

The `payload` macro can be used with the following directives:

- `path = mod::mod` – Specifies the module path for the payload if it is not directly imported at the location of `brec::generate!()`. This approach is not recommended (it is better to ensure payload visibility at the generator call site), but it is not inherently inefficient or unstable. However, using this method may make future code maintenance more difficult.
- `no_crc` – Disables CRC verification for the payload. Note that this does not remove the CRC field from the binary representation of the payload (specifically in `PayloadHeader`). The CRC field will still be present but filled with zeros, and no CRC calculation will be performed.
- `no_auto_crc` – Disables CRC verification for `payload(bincode)`, requiring a manual implementation of the `PayloadCrc` trait. This parameter is only relevant when using the `bincode` feature.
- `bincode` – available only when the bincode feature is enabled. It allows using any structure as a payload as long as it meets the requirements of the bincode crate, i.e., it implements serde serialization and deserialization. Please note that bincode has a number of limitations, which you can review in its official documentation.

## Packets

Users do not need to define possible packet types since any combination of blocks (up to 255) and a single optional payload constitutes a valid packet.

```ignore
brec::generate!();

let my_packet = Packet::new(
    // You are limited to 255 blocks per packet.
    vec![
        Block::MyBlockA(MyBlockA::default()),
        Block::MyBlockC(MyBlockC::default())
    ],
    // Note: payload is optional
    Some(Payload::MyPayloadA(MyPayloadA::default()))
);
```

### Packet Constraints

- A packet can contain **0 to 255 blocks**.
- A packet can include **0 or 1 payload**.

**Warning!** In most cases, having 1-5 blocks per packet is more than sufficient. A significant number of blocks can lead to an increase in compilation time but will not affect the performance of the compiled code. Therefore, if compilation time is a critical factor, it is recommended to avoid a large number of blocks in packets. 

To clarify, **runtime performance is not affected**, but the compilation time increases because the compiler has to generate multiple implementations for generic types used in `PacketDef` (an internal `brec` structure).

### Packet Trait Implementations

A `Packet` can be used as a standalone unit for data exchange. It implements the following traits:

| Trait                 | Method | Return Type | Description |
|-----------------------|--------|-------------|-------------|
| `ReadFrom`           | `read<T: std::io::Read>(buf: &mut T)` | `Result<Self, Error>` | Attempts to read a packet from a source. |
| `TryReadFrom`        | `try_read<T: std::io::Read + std::io::Seek>(buf: &mut T)` | `Result<ReadStatus<Self>, Error>` | Attempts to read a packet, but if data is insufficient, it returns a corresponding read status instead of an error. Also, moves the source’s position only upon successful reading; otherwise, it remains unchanged. |
| `TryReadFromBuffered` | `try_read<T: std::io::BufRead>(reader: &mut T)` | `Result<ReadStatus<Self>, Error>` | Identical to `TryReadFrom`. |
| `WriteMutTo`         | `write<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<usize>` | Equivalent to the standard `write` method, returning the number of bytes written. Does not guarantee that data is flushed to the output, so calling `flush` is required if such guarantees are needed. |
| `WriteMutTo`         | `write_all<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<()>` | Equivalent to the standard `write_all` method. |
| `WriteVectoredMutTo` | `slices(&mut self)` | `std::io::Result<IoSlices>` | Returns the binary representation of the packet as slices. |
| `WriteVectoredMutTo` | `write_vectored<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<usize>` | Attempts a vectored write of the packet (analogous to the standard `write_vectored`). |
| `WriteVectoredMutTo` | `write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T)` | `std::io::Result<()>` | Attempts a vectored write of the packet (analogous to the standard `write_vectored_all`). |

### Packet Filtering

`Packet` provides a highly useful method: 

```ignore
filtered<R: std::io::Read + std::io::Seek>(
    reader: &mut R, 
    rules: &Rules
) -> Result<LookInStatus<Packet>, Error>
```

This method allows you to "peek" into a packet before processing the payload, which can significantly improve performance when filtering specific packets.

## Code Generation

`brec` generates code in two stages:

- When the `#[block]` macro is used, it generates code specific to the corresponding block.  
  The same applies to the `#[payload]` macro, which generates code related to a specific payload type.

- However, the protocol also requires unified types such as `enum Block` (enumerating all user-defined blocks)  
  and `enum Payload` (enumerating all payloads). These general-purpose enums allow the system to handle  
  various blocks and payloads dynamically.

To generate this unified code, the `generate!()` macro must be invoked:

```ignore
pub use blocks::*;
pub use payloads::*;

brec::generate!();
```

This macro must be called exactly **once per crate** and is responsible for:

- Implementing required `brec` traits for all user-defined `Block` types
- Implementing required `brec` traits for all user-defined `Payload` types
- Generating unified enums for blocks: `enum Block { ... }`
- Generating unified enums for payloads: `enum Payload { ... }`
- Exporting several convenience type aliases to simplify usage

### Generated Aliases
The macro defines the following aliases to reduce verbosity when using `brec` types:

| Alias                    | Expanded to                                                                 |
|-------------------------|------------------------------------------------------------------------------|
| `Packet`                | `PacketDef<Block, Payload, Payload>`                                        |
| `PacketBufReader<'a, R>`| `PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>`     |
| `Rules<'a>`             | `RulesDef<Block, BlockReferred<'a>, Payload, Payload>`                      |
| `Rule<'a>`              | `RuleDef<Block, BlockReferred<'a>, Payload, Payload>`                       |
| `RuleFnDef<D, S>`       | `RuleFnDef<D, S>`                                                            |
| `Storage<S>`            | `StorageDef<S, Block, BlockReferred<'static>, Payload, Payload>`            |

These aliases make it easier to work with generated structures and remove the need to repeat generic parameters.

### Required Build Script

To enable this macro, you **must** include a `build.rs` file with the following content:
```ignore
    brec::build_setup();
```
This step ensures the code generator runs during build and provides all required metadata.

### Usage Constraints

- The macro **must only be called once** per crate. Calling it more than once will result in compilation errors due to duplicate types and impls.
- The macro **must see all relevant types** (`Block`, `Payload`) in scope. You must ensure they are visible in the location where you call the macro.

### Visibility Requirements

Ensure that all blocks and payloads are imported at the location where the macro is used:
```ignore
pub use blocks::*;
pub use payloads::*;

brec::generate!();
```

### Parameters

The macro can be used with the following parameters:

- `no_default_payload` – Disables the built-in payloads (`String` and `Vec<u8>`).  
  This has no impact on runtime performance but may slightly improve compile times and reduce binary size.

- `payloads_derive = "Trait"` –  
  By default, `brec` automatically collects all `derive` attributes that are common across user-defined payloads
  and applies them to the generated `Payload` enum.  
  This parameter allows you to **manually** specify additional derives for the `Payload` enum—useful if you are
  only using the built-in payloads (`String`, `Vec<u8>`) and do not define custom ones.

For example,

```ignore
pub use blocks::*;

// You don't define any custom payloads and only want to use the built-in ones (`String`, `Vec<u8>`)
brec::generate!(payloads_derive = "Debug, Clone");
```

```ignore
pub use blocks::*;

// You don't define any payloads and explicitly disable the built-in ones
brec::generate!(no_default_payload);
```

If the user **fully disables** payload support (as in the example above),
the macro will **not generate any packet-related types** (see *Generated Aliases*).

# Protocol Tools

## Reading Mixed and Mono Streams

To read from a data source, `brec` includes the `PacketBufReader<R: std::io::Read>` tool (available after code generation by calling `brec::generate!()`). `PacketBufReader` ensures safe reading from both **pure `brec` message streams** and **mixed data streams** (containing both `brec` messages and arbitrary data).

Below is an example of reading all `brec` messages from a stream while counting the number of "junk" bytes (i.e., data that is not a `brec` message):

```ignore
fn reading<R: std::io::Read>(source: &mut R) -> std::io::Result<(Vec<Packet>, usize)> {
    let mut packets: Vec<Packet> = Vec::new();
    let mut reader: PacketBufReader<_> = PacketBufReader::new(source);
    let ignored: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let ignored_inner = ignored.clone();
    
    reader
        .add_rule(Rule::Ignored(brec::RuleFnDef::Dynamic(Box::new(
            move |bytes: &[u8]| {
                ignored_inner.fetch_add(bytes.len(), Ordering::SeqCst);
            },
        ))))
        .unwrap();
    
    loop {
        match reader.read() {
            Ok(next) => match next {
                NextPacket::Found(packet) => packets.push(packet),
                NextPacket::NotFound => {
                    // Data will be refilled on the next call
                }
                NextPacket::NotEnoughData(_needed) => {
                    // Data will be refilled on the next call
                }
                NextPacket::NoData => {
                    break;
                }
                NextPacket::Skipped => {
                    //
                }
            },
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err.to_string(),
                ));
            }
        };
    }
    Ok((packets, ignored.load(Ordering::SeqCst)))
}
```

### Key Features of `PacketBufReader`
- If there is **insufficient data** (`NextPacket::NotEnoughData`), `PacketBufReader` will attempt to load more data on each subsequent call to `read()`.  
- If **no `brec` data is found** in the current `read()` iteration (`NextPacket::NotFound`), `PacketBufReader` will also attempt to load more data on each subsequent `read()`.  

Thus, `PacketBufReader` **automatically manages data loading**, removing the need for users to implement their own data-fetching logic.

### `NextPacket` Read Statuses

| Status                    | Description | Can Continue Reading? |
|---------------------------|-------------|------------------------|
| `NextPacket::Found`       | A packet was successfully found and returned. | ✅ Yes |
| `NextPacket::NotFound`    | No packets were found in the current read iteration. | ✅ Yes |
| `NextPacket::NotEnoughData` | A packet was detected, but there is not enough data to read it completely. | ✅ Yes |
| `NextPacket::Skipped`     | A packet was detected but skipped due to filtering rules. | ✅ Yes |
| `NextPacket::NoData`      | No more data can be retrieved from the source. | ❌ No |

After receiving `NextPacket::NoData`, further calls to `read()` are meaningless, as `PacketBufReader` has exhausted all available data from the source.

### Custom Filtering Rules in `PacketBufReader`

Another key feature of `PacketBufReader` is that users can define **custom rules** to be applied during data reading. These rules can be updated dynamically between `read()` calls using `add_rule` and `remove_rule`.

| Rule                   | Available Data                      | Description |
|------------------------|--------------------------------------|-------------|
| `Rule::Ignored`        | `&[u8]`                              | Triggered when data not related to `brec` messages is encountered. Provides a byte slice of the unrelated data. |
| `Rule::FilterByBlocks` | `&[BlockReferred<'a>]`              | Triggered when a packet is found and its blocks have been partially parsed. If blocks contain slices, no copying is performed — `BlockReferred` will hold references instead. At this stage, the user can decide whether to proceed with parsing the "heavy" part (i.e., the payload) or skip the packet. |
| `Rule::FilterByPayload`| `&[u8]`                              | Allows "peeking" into the payload bytes before deserialization. This is especially useful if the payload is, for example, a string — enabling scenarios like substring search. |
| `Rule::Filter`         | `&Packet`                            | Triggered after the packet is fully parsed, giving the user a final chance to accept or reject the packet. |

The rules `Rule::FilterByBlocks` and `Rule::FilterByPayload` are particularly effective at improving performance, as they allow you to skip the most expensive part — parsing the payload — if the packet is not needed.

## `brec` Message Storage

In addition to stream reading, `brec` provides a tool for storing packets and accessing them efficiently — `Storage<S: std::io::Read + std::io::Write + std::io::Seek>` (available after invoking `brec::generate!()`).

| Method                                | Description |
|--------------------------------------|-------------|
| `insert(&mut self, packet: Packet)`  | Inserts a packet into the storage. |
| `add_rule(&mut self, rule: Rule)`    | Adds a filtering rule. |
| `remove_rule(&mut self, rule: RuleDefId)` | Removes a filtering rule. |
| `count(&self)` | Returns the number of records currently stored. |
| `iter(&mut self)`                    | Returns an iterator over the storage. This method does not apply filters, even if previously added. |
| `filtered(&mut self)`                | Returns an iterator with filters applied (if any were set via `add_rule`). The filtering rules used in `Storage` are identical to those used in `PacketBufReader`. |
| `nth(&mut self, nth: usize)`         | Attempts to read the packet at the specified index. Note that this method does not apply any filtering, even if filters have been previously defined. |
| `range(&mut self, from: usize, len: usize)` | Returns an iterator over a given range of packets. |
| `range_filtered(&mut self, from: usize, len: usize)` | Returns an iterator over a range of packets with filters applied (if previously set via `add_rule`). |

Filtering by blocks or payload improves performance by allowing the system to avoid fully parsing packets unless necessary.

### Storage Layout and Slot Design

The core design of `Storage` is based on how it organizes packets internally:
- Packets are not stored sequentially but are grouped into **slots**, with **500 packets per slot**.
- Each slot stores metadata about packet positions in the file and includes a **CRC** for slot validation, which makes the storage robust against corruption.
- Thanks to the slot metadata, `Storage` can **quickly locate packets by index** or **return a packet range efficiently**.

As previously mentioned, each slot maintains its own **CRC** to ensure data integrity. However, even if the storage file becomes corrupted and `Storage` can no longer operate reliably, packets remain accessible in a **manual recovery mode**. For example, you can use `PacketBufReader` to scan the file, ignoring slot metadata and extracting intact packets sequentially.

# Protocol Specification

## Block

A block supports fields of the following types:

| Type   | Size in Binary Format (bytes) |
|--------|-------------------------------|
| u8     | 1                             |
| u16    | 2                             |
| u32    | 4                             |
| u64    | 8                             |
| u128   | 16                            |
| i8     | 1                             |
| i16    | 2                             |
| i32    | 4                             |
| i64    | 8                             |
| i128   | 16                            |
| f32    | 4                             |
| f64    | 8                             |
| bool   | 1                             |
| [u8; n] | n                            |

Any structure marked with the `block` macro will have the following extended representation in binary format:

| Field                     | Type             |
|---------------------------|------------------|
| Signature                 | [u8; 4]          |
| User-defined fields       | Available types  |
| CRC                       | [u8; 4]          |

Thus, the total binary length of a block is calculated as:

```ignore
length = 4 (Signature) + Block's Fields Length + 4 (CRC)
```

The block signature is generated automatically based on its name (including the module path) and the names of all its fields. The signature hash is computed using a 32-bit algorithm.

The block's CRC (32-bit) is generated based on the values of user-defined fields, excluding the signature and the CRC itself.

## Payload

Any data type that implements the `PayloadEncode` and `PayloadDecode<T>` traits can be used as a payload. These traits handle the conversion of a payload to bytes and its unpacking from bytes, respectively. When serializing a payload into binary format, `brec` automatically adds a `PayloadHeader` to each payload.

### `PayloadHeader` Structure

| Field                 | Size           | Description |
|-----------------------|---------------|-------------|
| Signature Length     | 1 byte        | Length of the signature: 4, 8, 16, 32, 64, or 128 bytes |
| Signature           | 4 to 128 bytes | Unique signature of the payload |
| CRC Length         | 1 byte        | Length of the CRC: 4, 8, 16, 32, 64, or 128 bytes |
| CRC                | 4 to 128 bytes | CRC checksum of the payload |
| Payload Body Length | 4 bytes       | Length of the payload body (`u32`) |

As seen in the `PayloadHeader` structure, it does not have a fixed size since the lengths of the signature and CRC can vary. For example, when using the `bincode` feature, both the signature and CRC lengths are set to 4 bytes (32 bits). However, users can implement their own versions with different lengths.

Thus, any payload in binary format is represented as follows:

| Component       | Size           | Description |
|---------------|---------------|-------------|
| `PayloadHeader` | 14 - 262 bytes | Header containing metadata about the payload |
| Payload's body | ---           | Payload data encoded using `PayloadEncode` |

The CRC of the payload is generated based on the bytes produced by `PayloadEncode`. This introduces some constraints on CRC verification since `brec` does not restrict the types of data used in a payload. If a payload contains data types that do not guarantee a strict byte sequence, CRC verification will always fail due to byte order variations. As a result, extracting the payload from the data stream will become impossible.

A simple example of such a situation is a `HashMap`, which does not guarantee a consistent field order when reconstructed. For instance, defining a payload like this:

```ignore
#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyPayloadB {
    items: HashMap<String, String>,
}
```

would make it impossible to extract this payload, as the CRC would always be different (except when the number of keys in the map is ≤ 1). This issue can be resolved in several ways:

- The simplest approach is to avoid using "unstable" data types and instead choose one that guarantees a fixed byte sequence.
- Disable CRC verification for this specific payload by using the `no_crc` directive: `#[payload(bincode, no_crc)]`.
- Disable automatic CRC calculation and implement the `PayloadCrc` trait manually for the specific payload. Automatic CRC calculation can be disabled using the `no_auto_crc` directive: `#[payload(bincode, no_auto_crc)]`.

## Packet

A packet serves as a container for storing a set of blocks and optionally a single payload. Each packet includes its own header, `PacketHeader`:

### `PacketHeader` Structure

| Field                   | Size   | Description |
|-------------------------|--------|-------------|
| Signature              | 8 bytes  | Static packet signature |
| Size                   | 8 bytes  | Total size of the packet (excluding the `PacketHeader`) |
| Block's length         | 8 bytes  | Total length of all blocks (including signatures and CRC) in the packet |
| Payload existence flag | 1 byte   | `0` - packet without payload; `1` - packet contains a payload |
| CRC                    | 4 bytes  | CRC of the `PacketHeader` (not the entire packet, only the header) |

Thus, in binary format, a packet is structured as follows:

| Component       | Size    | Count |
|---------------|--------|------|
| `PacketHeader` | 29 bytes | 1 |
| `Block`        | ---    | 0 to 255 |
| `Payload`      | ---    | 0 or 1 |

# Ensuring `brec` Stability

The stability of `brec` is ensured through two levels of testing.

## Functional Testing

To test reading, writing, parsing, filtering, and other functions, `brec` uses `proptest` to generate **random values** for predefined (structurally consistent) blocks and payloads. Blocks and payloads are tested **both separately and in combination** as part of packet testing. 

Packets are constructed with **randomly generated blocks and payloads**. Additionally, the ability of `brec` tools to **reliably read and write randomly generated blocks** is also tested, specifically focusing on `Storage<S: std::io::Read + std::io::Write + std::io::Seek>` and `PacketBufReader`.

In total, **over 40 GB of test data** is generated for this type of testing.

## Macro Testing

To validate the behavior of the `block` and `payload` macros, `brec` also uses `proptest`, but this time it **not only generates random data but also randomly constructs block and payload structures**.

Each randomly generated set of structures is saved as a separate crate. After generating these test cases, each one is **compiled and executed** to ensure stability. Specifically, all randomly generated packets **must be successfully encoded and subsequently decoded without errors**.

## Performance and Efficiency

To evaluate the performance of the protocol, the following data structure is used:

```ignore
pub enum Level {
    Err,
    Warn,
    Info,
    Debug,
}

pub enum Target {
    Server,
    Client,
    Proxy,
}

#[block]
pub struct Metadata {
    pub level: Level,
    pub target: Target,
    pub tm: u64,
}
```

Note: Conversion of `Level` and `Target` into `u8` is required but omitted here for brevity.

Each packet consists of a `Metadata` block and a `String` payload. Data is randomly generated, and a special "hook" string is inserted randomly into some messages for use in filtering tests.

### Test Description

- **Storage**: Data is written using the `brec` storage API — `Storage<S: std::io::Read + std::io::Write + std::io::Seek>` — and then read back using the same interface.
- **Binary Stream**: Data is written to the file as a plain stream of packets, without slots or metadata. Then it is read using `PacketBufReader`.
- **Streamed Storage**: Data is written using `Storage`, but read using `PacketBufReader`, which ignores slot metadata (treating it as garbage).
- **Plain Text**: Raw text lines are written to the file, separated by `\n`.
- **JSON**: The structure shown above is serialized to JSON using `serde_json` and written as one JSON object per line. During reading, each line is deserialized back to the original structure.

Each test is run in two modes:
- **Reading** — reading all available data.
- **Filtering** — reading only records that match specific criteria: logs of type "error" and containing a search hook in the payload.

**Plain Text** is used as a baseline due to its minimal overhead — raw sequential file reading with no parsing or decoding.  
However, `brec` performance is more meaningfully compared with **JSON**, which also involves deserialization.  
JSON is considered a strong baseline due to its wide use and mature, highly optimized parser.

### Important Notes

- For fairness, **CRC checks are enabled** for all `brec` component. CRC is calculated for blocks, payloads, and slots (in the case of storage).
- Each test is repeated multiple times to produce averaged values (`Iterations` column).

### Test Results

| Test             | Mode      | Size    | Rows       | Time (ms) | Iterations |
|------------------|-----------|---------|------------|-----------|------------|
| Storage          | Filtering | 908 Mb  | 140,000     | 612       | 10         |
| Storage          | Reading   | 908 Mb  | 1,000,000   | 987       | 10         |
| JSON             | Reading   | 919 Mb  | 1,000,000   | 597       | 10         |
| JSON             | Filtering | 919 Mb  | 140,000     | 608       | 10         |
| Binary Stream    | Reading   | 831 Mb  | 1,000,000   | 764       | 10         |
| Binary Stream    | Filtering | 831 Mb  | 140,000     | 340       | 10         |
| Plain Text       | Reading   | 774 Mb  | 1,000,000   | 247       | 10         |
| Plain Text       | Filtering | 774 Mb  | 150,000     | 276       | 10         |
| Streamed Storage | Filtering | 908 Mb  | 140,000     | 355       | 10         |
| Streamed Storage | Reading   | 908 Mb  | 1,000,000   | 790       | 10         |

### Observations

- **Plain text** is the fastest format by nature and serves as a baseline.
- **Storage** gives the slowest reading time in full-scan mode — which is expected due to CRC verification and slot parsing.
- However, when **filtering is enabled**, storage is **only 4ms slower than JSON**, which is a **negligible difference**, especially considering that storage data is CRC-protected and recoverable.
- If the storage file is damaged, packets can still be recovered using `PacketBufReader`, even if the slot metadata becomes unreadable.
- **Binary stream mode** (stream writing and reading with `PacketBufReader`) shows exceptional filtering performance — nearly **twice as fast as JSON** — and even full reading is only slightly slower than JSON (~167ms on 1 GB), which is not significant in most scenarios.

This efficiency is possible because `brec`'s architecture allows it to skip unnecessary work. In contrast to JSON, where every line must be deserialized, `brec` can **evaluate blocks before parsing payloads**, leading to better filtering performance.
