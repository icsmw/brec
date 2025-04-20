
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

```rust
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

```rust
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
