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

```rust
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

```rust
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

```rust
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
