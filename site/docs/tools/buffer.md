## Reading Mixed and Mono Streams

To read from a data source, `brec` includes the `PacketBufReader<R: std::io::Read>` tool (available after code generation by calling `brec::generate!()`). `PacketBufReader` ensures safe reading from both **pure `brec` message streams** and **mixed data streams** (containing both `brec` messages and arbitrary data).

Below is an example of reading all `brec` messages from a stream while counting the number of "junk" bytes (i.e., data that is not a `brec` message). The statistics live in the reader workflow context, not in the payload protocol context:

```rust
#[derive(Default)]
struct ReaderStats {
    ignored_bytes: usize,
}

fn reading<R: std::io::Read>(source: &mut R) -> std::io::Result<(Vec<Packet>, usize)> {
    let mut packets: Vec<Packet> = Vec::new();
    let mut reader = PacketBufReader::with_context(source, ReaderStats::default());
    let mut ctx = brec::default_payload_context();
    
    reader
        .add_rule(Rule::IgnoredControl(brec::RuleFnDef::Static(
            |bytes: &[u8], stats: &mut ReaderStats| {
                stats.ignored_bytes += bytes.len();
                Ok(brec::IgnoredAction::Continue)
            },
        )))
        .unwrap();
    
    loop {
        match reader.read(&mut ctx) {
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
    let ignored = reader.into_context().ignored_bytes;
    Ok((packets, ignored))
}
```

### Key Features of `PacketBufReader`
- If there is **insufficient data** (`NextPacket::NotEnoughData`), `PacketBufReader` will attempt to load more data on each subsequent call to `read(ctx)`.  
- If **no `brec` data is found** in the current `read(ctx)` iteration (`NextPacket::NotFound`), `PacketBufReader` will also attempt to load more data on each subsequent `read(ctx)`.  

Thus, `PacketBufReader` **automatically manages data loading**, removing the need for users to implement their own data-fetching logic. The protocol context must still be supplied on each `read(ctx)` call. The reader also applies `ProtocolSchema::MAX_PACKET_LEN` before buffering a detected packet body.

### `NextPacket` Read Statuses

| Status                    | Description | Can Continue Reading? |
|---------------------------|-------------|------------------------|
| `NextPacket::Found`       | A packet was successfully found and returned. | ✅ Yes |
| `NextPacket::NotFound`    | No packets were found in the current read iteration. | ✅ Yes |
| `NextPacket::NotEnoughData` | A packet was detected, but there is not enough data to read it completely. | ✅ Yes |
| `NextPacket::Skipped`     | A packet was detected but skipped due to filtering rules. | ✅ Yes |
| `NextPacket::NoData`      | No more data can be retrieved from the source. | ❌ No |

After receiving `NextPacket::NoData`, further calls to `read(ctx)` are meaningless, as `PacketBufReader` has exhausted all available data from the source.

### Custom Filtering Rules in `PacketBufReader`

Another key feature of `PacketBufReader` is that users can define **custom rules** to be applied during data reading. These rules can be updated dynamically between `read(ctx)` calls using `add_rule` and `remove_rule`.

| Rule                   | Available Data                      | Description |
|------------------------|--------------------------------------|-------------|
| `Rule::Ignored`        | `&[u8]`                              | Triggered when data not related to `brec` messages is encountered. Provides a byte slice of the unrelated data. |
| `Rule::IgnoredControl` | `&[u8]`, `&mut WorkflowCtx`          | Like `Ignored`, but can update reader-owned workflow state and stop scanning by returning `IgnoredAction::Stop`. |
| `Rule::NextPacket`     | `&NextPacket`, `&mut WorkflowCtx`    | Triggered immediately before a non-error `NextPacket` result is returned. Useful for diagnostics and stream health metrics. |
| `Rule::Prefilter`      | `PeekedBlocks<'a>`                   | Triggered when a packet is found and its blocks have been partially parsed in zero-copy mode. This is the cheapest place to decide whether the payload should be parsed at all. |
| `Rule::FilterPayload`  | `&[u8]`                              | Allows peeking into the payload bytes before deserialization. This is especially useful if the payload is, for example, a string - enabling scenarios like substring search. |
| `Rule::FilterPacket`   | `&Packet`                            | Triggered after the packet is fully parsed, giving the user a final chance to accept or reject the packet. |

`PeekedBlocks` is the main user-facing facade for cheap prefiltering. It hides the low-level `BlockReferred<'a>` representation while still allowing advanced access through `PeekedBlock::as_referred()` and `PeekedBlocks::as_slice()` when needed.

The rules `Rule::Prefilter` and `Rule::FilterPayload` are particularly effective at improving performance, as they allow you to skip the most expensive part - parsing the payload - if the packet is not needed.

### Workflow Context and Bad Bytes

`PacketBufReader` can own a workflow context that is separate from the protocol context passed to `read(ctx)`.

- protocol context is for payload encode/decode logic;
- workflow context is for reader-side diagnostics and control decisions.

Use `PacketBufReader::with_context(source, stats)` when you want rules to update custom state such as ignored byte counters, packet counters, or stream health scores.

```rust
#[derive(Default)]
struct ReaderStats {
    ignored_bytes: usize,
    found_packets: usize,
}

let mut reader = PacketBufReader::with_context(source, ReaderStats::default());

reader.add_rule(Rule::IgnoredControl(brec::RuleFnDef::Static(
    |bytes, stats| {
        stats.ignored_bytes += bytes.len();

        if stats.ignored_bytes > 64 * 1024 {
            Ok(brec::IgnoredAction::Stop)
        } else {
            Ok(brec::IgnoredAction::Continue)
        }
    },
)))?;

reader.add_rule(Rule::NextPacket(brec::RuleFnDef::Static(
    |next, stats| {
        if matches!(next, NextPacket::Found(_)) {
            stats.found_packets += 1;
        }
        Ok(())
    },
)))?;
```

Returning `IgnoredAction::Stop` from `Rule::IgnoredControl` aborts reading with `Error::IgnoredDataRejected`. This is the recommended pattern for pure network protocols where arbitrary bytes before a packet are suspicious: count or inspect ignored bytes, then close the connection once the stream exceeds your policy. For file recovery or log scanning, return `IgnoredAction::Continue` and keep the collected diagnostics instead.

### Recommended Filtering Flow

In practice, filtering is most effective when it is performed in three stages:

1. `Rule::Prefilter`
2. `Rule::FilterPayload`
3. `Rule::FilterPacket`

This staged approach minimizes unnecessary work:

- `Prefilter` avoids decoding the payload at all.
- `FilterPayload` avoids full payload deserialization when raw bytes are enough.
- `FilterPacket` is the final, most expensive decision point.

### Using `PeekedBlocks`

`PeekedBlocks<'a>` is the main facade for cheap block-based filtering. It intentionally exposes operations in terms of user-defined block types instead of forcing manual matching on `BlockReferred<'a>`.

Available methods:

| Method | Description |
|--------|-------------|
| `has::<T>()` | Returns `true` if at least one block of type `T` is present. |
| `get::<T>()` | Returns the first block of type `T`. |
| `find::<T, _>(predicate)` | Returns the first block of type `T` that satisfies the predicate. |
| `iter_as::<T>()` | Iterates only over blocks of type `T`. |
| `iter()` | Iterates over all blocks as `PeekedBlock`. |
| `nth(index)` | Returns the block view at the specified position. |
| `as_slice()` | Returns the underlying slice of referred blocks. This is the advanced escape hatch. |

Available methods on `PeekedBlock<'a>`:

| Method | Description |
|--------|-------------|
| `as_type::<T>()` | Attempts to view the current block as a concrete block type `T`. |
| `as_referred()` | Returns the underlying referred block. This is the advanced escape hatch. |

#### Example: fast prefilter by a block type

```rust
reader
    .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
        move |blocks| blocks.has::<Metadata>(),
    ))))
    .unwrap();
```

#### Example: find a specific block value

```rust
reader
    .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
        move |blocks| {
            blocks
                .find::<Metadata, _>(|meta| matches!(meta.level, Level::Err))
                .is_some()
        },
    ))))
    .unwrap();
```

#### Example: iterate only over one block type

```rust
reader
    .add_rule(Rule::Prefilter(brec::RuleFnDef::Dynamic(Box::new(
        move |blocks| {
            blocks
                .iter_as::<Metadata>()
                .any(|meta| matches!(meta.level, Level::Err))
        },
    ))))
    .unwrap();
```

Use `PeekedBlock::as_referred()` or `PeekedBlocks::as_slice()` only when:

- you need direct access to the generated low-level representation,
- the typed helpers are not expressive enough for your case,
- or you are building advanced integrations on top of `brec`.
