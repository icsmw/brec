## Performance and Efficiency

To evaluate the performance of the protocol, the following data structure is used:

```rust
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
