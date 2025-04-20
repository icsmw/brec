
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

```rust
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

```rust
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

```rust
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
