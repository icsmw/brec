## `brec` Message Storage

In addition to stream reading, `brec` provides a storage layer built around two generated types:

- `Writer<S>` - appends packets to storage
- `Reader<S>` - reads packets back with indexed access and filtering

Both become available after invoking `brec::generate!()`.

| Method                                | Description |
|--------------------------------------|-------------|
| `Writer::insert(&mut self, packet: Packet, ctx: &mut PayloadContext<'_>)` | Inserts a packet into the storage. |
| `Reader::add_rule(&mut self, rule: Rule)` | Adds a filtering rule. |
| `Reader::remove_rule(&mut self, rule: RuleDefId)` | Removes a filtering rule. |
| `Reader::count(&self)` | Returns the number of records currently stored. |
| `Reader::iter(&mut self, ctx: &mut PayloadContext<'_>)` | Returns an iterator over the storage. This method does not apply filters, even if previously added. |
| `Reader::filtered(&mut self, ctx: &mut PayloadContext<'_>)` | Returns an iterator with filters applied. The filtering rules are identical to those used in `PacketBufReader`. |
| `Reader::nth(&mut self, nth: usize, ctx: &mut PayloadContext<'_>)` | Attempts to read the packet at the specified index. This method does not apply filtering. |
| `Reader::range(&mut self, from: usize, len: usize, ctx: &mut PayloadContext<'_>)` | Returns an iterator over a given range of packets. |
| `Reader::range_filtered(&mut self, from: usize, len: usize, ctx: &mut PayloadContext<'_>)` | Returns an iterator over a range of packets with filters applied. |
| `Reader::seek(&mut self, packet: usize, ctx: &mut PayloadContext<'_>)` | Returns an iterator starting from the specified packet index. |
| `Reader::reload(&mut self)` | Reloads slot metadata and discovers packets appended after the reader was created. |

Filtering by blocks or payload improves performance by allowing the system to avoid fully parsing packets unless necessary.

### Storage Layout and Slot Design

The storage layer is based on how it organizes packets internally:
- Packets are not stored sequentially but are grouped into **slots**, with **500 packets per slot**.
- Each slot stores metadata about packet positions in the file and includes a **CRC** for slot validation, which makes the storage robust against corruption.
- Thanks to the slot metadata, `Reader` can **quickly locate packets by index** or **return a packet range efficiently**.

As previously mentioned, each slot maintains its own **CRC** to ensure data integrity. However, even if the storage file becomes corrupted and the slot metadata can no longer be trusted, packets remain accessible in a **manual recovery mode**. For example, you can use `PacketBufReader` to scan the file, ignore slot metadata, and extract intact packets sequentially.

## File Observation

When the `observer` feature is enabled, `brec` can watch a storage file and react to newly appended packets.

Two generated facades are available:

- `FileObserver` - callback-based consumption through `Subscription`
- `FileObserverStream` - Tokio stream of observer events

### Callback-based Observation

Use `FileObserver` when you want push-style handling through a subscription object:

```rust
struct MySubscription;

impl Subscription for MySubscription {
    fn on_update(&mut self, total: usize, added: usize) -> SubscriptionUpdate {
        let _ = (total, added);
        SubscriptionUpdate::Read
    }

    fn on_packet(&mut self, packet: Packet) {
        let _ = packet;
    }
}

let options = FileObserverOptions::new(path).subscribe(MySubscription);
let mut observer = FileObserver::new(options)?;
```

`Subscription` uses `on_*` callbacks:

- `on_update`
- `on_packet`
- `on_error`
- `on_stopped`
- `on_aborted`

### Stream-based Observation

Use `FileObserverStream` when you want pull-style integration in Tokio code:

```rust
use tokio_stream::StreamExt;

let mut stream = FileObserverStream::new(path)?;

while let Some(event) = stream.next().await {
    match event {
        brec::FileObserverEvent::Packet(packet) => {
            let _ = packet;
        }
        brec::FileObserverEvent::Update { total, added } => {
            let _ = (total, added);
        }
        brec::FileObserverEvent::Error(err) => {
            eprintln!("{err}");
        }
        brec::FileObserverEvent::Stopped(reason) => {
            let _ = reason;
            break;
        }
        brec::FileObserverEvent::Aborted => {
            break;
        }
    }
}
```

### Important Runtime Note

The observer integrates well with Tokio and exposes an async-friendly API, but at the low level it still relies on synchronous, blocking file I/O (`std::fs::File`, `Read`, `Seek`) through the storage reader.

In other words:

- async orchestration: yes
- true non-blocking disk access: no

This is the current design and should be kept in mind when embedding the observer into latency-sensitive async workflows.

## Locked file storage

When the locked_storage feature is enabled, you gain access to `FileStorage`, a file-backed wrapper around `WriterDef` with built-in file locking capabilities.

### How it works

When a new instance of `FileStorage` is created, a .lock file is created alongside the target storage file. An exclusive advisory lock is then applied to the .lock file using the fs4 crate.

This mechanism ensures that:

- Multiple `FileStorage` instances from the same or different processes will respect the lock
- Only one instance can access the underlying file at a time
- Lock contention can be resolved automatically via timeout and polling

**Important**: This is an advisory lock. It only prevents access for processes that respect the locking protocol (such as other brec-based tools). It does not prevent other unrelated tools, editors, or system calls from modifying the file.

### Coordinated Access

`FileStorage` supports:

- An optional timeout for acquiring the lock
- A customizable polling interval while waiting

This allows safe coordination in multi-process environments, without resorting to global OS-level locks.

### Example 

```
FileStorage::new(
    &filename,                          // Path to file
    Some(Duration::from_millis(100)),   // Timeout
    None,                               // Interval to check lock state (if file was locked and we are waiting)
);

// Or with options
FileStorageOptions::new(filename)
    .timeout(Duration::from_millis(300))
    .interval(Duration::from_millis(50))
    .open();
```
