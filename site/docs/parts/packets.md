
Users do not need to define possible packet types since any combination of blocks (up to 255) and a single optional payload constitutes a valid packet.

```rust
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

```rust
filtered<R: std::io::Read + std::io::Seek>(
    reader: &mut R, 
    rules: &Rules
) -> Result<LookInStatus<Packet>, Error>
```

This method allows you to "peek" into a packet before processing the payload, which can significantly improve performance when filtering specific packets.
