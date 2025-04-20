## Reading Mixed and Mono Streams

To read from a data source, `brec` includes the `PacketBufReader<R: std::io::Read>` tool (available after code generation by calling `brec::generate!()`). `PacketBufReader` ensures safe reading from both **pure `brec` message streams** and **mixed data streams** (containing both `brec` messages and arbitrary data).

Below is an example of reading all `brec` messages from a stream while counting the number of "junk" bytes (i.e., data that is not a `brec` message):

```rust
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
