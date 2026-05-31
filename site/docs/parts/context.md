# Protocol Context

`brec` separates packet structure from payload runtime state.

Blocks are context-free. Payloads may require extra runtime data during:

- encoding
- decoding
- size calculation
- packet read/write operations

This runtime data is called `ProtocolContext`.

## Why It Exists

Some payloads cannot be encoded or decoded from bytes alone.

Typical examples:

- encryption settings
- decryption settings
- external lookup state
- user-defined runtime options

For the built-in crypto integration that uses this mechanism, see [Crypt](../features/crypt.md).

Instead of passing an arbitrary generic options type through the whole API, `brec` binds a context type and protocol size limits to the generated protocol through `ProtocolSchema`.

## Core Idea

Each payload defines:

```rust
pub trait ProtocolSchema {
    type Context<'a>;

    const MAX_PAYLOAD_LEN: u32;
    const MAX_PACKET_LEN: u64;
    const INITIAL_PACKET_BUFFER_CAPACITY: usize;
}
```

All payload-specific traits then receive this context explicitly:

```rust
fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>>;
fn decode(buf: &[u8], ctx: &mut Self::Context<'_>) -> std::io::Result<Self>;
fn size(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<u64>;
```

As a result:

- block logic stays clean
- payload logic can use runtime state when needed
- packet and storage APIs stay explicit about where context is consumed
- packet and payload readers share the same configured size limits

`MAX_PAYLOAD_LEN` is the maximum accepted payload body length. `MAX_PACKET_LEN` is the maximum accepted packet body length, excluding `PacketHeader`. `INITIAL_PACKET_BUFFER_CAPACITY` is the initial allocation used by `PacketBufReader`; it is a performance hint, not a validity boundary.

For generated protocols these values are configured through `brec::generate!()`. See [Code Generation](../code_generation.md#parameters).

## Default Context

If a payload does not need any runtime state, use the default context:

```rust
type Context<'a> = brec::DefaultProtocolContext;
```

`DefaultProtocolContext` is just `()`.

## Generated Context

When you use `brec::generate!()`, the macro generates a crate-local `ProtocolContext<'a>`.

If there are no custom context entries, it becomes:

```rust
pub type ProtocolContext<'a> = ();
```

If custom context types exist, `generate!()` builds an enum:

```rust
pub enum ProtocolContext<'a> {
    None,
    MyOptions(&'a mut MyOptions),
}
```

The working examples are available in the `examples` directory of the repository.

## Declaring a Custom Context Type

Mark a type with `#[payload(ctx)]`:

```rust
use brec::payload;

#[payload(ctx)]
pub struct MyOptions {
    pub prefix: String,
}
```

This type is not treated as a regular payload. It is collected only to build `ProtocolContext<'a>`.

In other words, `#[payload(ctx)]` means:

- do not generate a normal payload variant for this type
- do generate a matching `ProtocolContext` enum variant
- pass this value by mutable reference during payload operations

## Manual Payload Implementation with Context

If you implement payload traits manually, you can use the generated context and extract your runtime state.

Important: `#[payload]` already generates part of the boilerplate for the type itself.

In the common manual case you only implement the payload-specific logic such as:

- `PayloadEncode`
- `PayloadEncodeReferred`
- `PayloadDecode<T>`
- `PayloadSize`
- optionally `PayloadCrc`

You do not need to manually reimplement the basic glue that `#[payload]` already provides for the example shape below.

Example:

```rust
use brec::{PayloadCrc, PayloadDecode, PayloadEncode, PayloadEncodeReferred, PayloadSize};

#[payload(ctx)]
pub struct MyOptions {
    pub prefix: String,
}

impl MyOptions {
    fn extract_prefix<'a>(ctx: &'a mut crate::ProtocolContext<'_>) -> std::io::Result<&'a str> {
        match ctx {
            crate::ProtocolContext::MyOptions(options) => Ok(options.prefix.as_str()),
            crate::ProtocolContext::None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "MyPayload expects ProtocolContext::MyOptions",
            )),
        }
    }
}

#[payload]
pub struct MyPayload {
    pub value: String,
}

impl PayloadEncode for MyPayload {
    fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
        let prefix = MyOptions::extract_prefix(ctx)?;
        Ok(format!("{}{}", prefix, self.value).into_bytes())
    }
}

impl PayloadEncodeReferred for MyPayload {
    fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
        Ok(None)
    }
}

impl PayloadDecode<MyPayload> for MyPayload {
    fn decode(buf: &[u8], ctx: &mut Self::Context<'_>) -> std::io::Result<MyPayload> {
        let prefix = MyOptions::extract_prefix(ctx)?.to_owned();

        let value = String::from_utf8(buf.to_vec())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        let value = value
            .strip_prefix(&prefix)
            .unwrap_or(&value)
            .to_owned();

        Ok(MyPayload { value })
    }
}

impl PayloadSize for MyPayload {
    fn size(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<u64> {
        Ok(PayloadEncode::encode(self, ctx)?.len() as u64)
    }
}

impl PayloadCrc for MyPayload {}
```

This is the same pattern used in the real example. The full example can be found in the repository under `examples/ctx`.

## Passing Context into Readers and Writers

Context is supplied at the operation boundary.

Examples:

```rust
let packet = Packet::new(
    vec![Block::MyBlock(MyBlock { id: 1 })],
    Some(Payload::MyPayload(MyPayload {
        value: "hello".to_owned(),
    })),
);
let mut ctx = ProtocolContext::MyOptions(&mut options);
writer.insert(packet, &mut ctx)?;
```

```rust
let mut ctx = ProtocolContext::MyOptions(&mut options);
for packet in reader.iter(&mut ctx) {
    let packet = packet?;
    // ...
}
```

```rust
let mut ctx = ProtocolContext::MyOptions(&mut options);
match packet_reader.read(&mut ctx)? {
    NextPacket::Found(packet) => {
        let _ = packet;
    }
    _ => {}
}
```

The key idea is:

- payload type owns the serialization logic
- context is created by the caller
- reader and writer APIs receive that context explicitly per operation

## Practical Rule

Use context only for runtime state that genuinely belongs to payload processing.

Good candidates:

- crypto options
- decode-time lookup state
- encode/decode feature flags

Bad candidates:

- block-level concerns
- global application state unrelated to payload bytes
- values that should be stored inside the payload itself
