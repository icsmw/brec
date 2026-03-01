## `brec` Message Storage

In addition to stream reading, `brec` provides a storage layer built around two generated types:

- `Writer<S>` — appends packets to storage
- `Reader<S>` — reads packets back with indexed access and filtering

Both become available after invoking `brec::generate!()`.

| Method                                | Description |
|--------------------------------------|-------------|
| `Writer::insert(&mut self, packet: Packet)` | Inserts a packet into the storage. |
| `Reader::add_rule(&mut self, rule: Rule)` | Adds a filtering rule. |
| `Reader::remove_rule(&mut self, rule: RuleDefId)` | Removes a filtering rule. |
| `Reader::count(&self)` | Returns the number of records currently stored. |
| `Reader::iter(&mut self)` | Returns an iterator over the storage. This method does not apply filters, even if previously added. |
| `Reader::filtered(&mut self)` | Returns an iterator with filters applied. The filtering rules are identical to those used in `PacketBufReader`. |
| `Reader::nth(&mut self, nth: usize)` | Attempts to read the packet at the specified index. This method does not apply filtering. |
| `Reader::range(&mut self, from: usize, len: usize)` | Returns an iterator over a given range of packets. |
| `Reader::range_filtered(&mut self, from: usize, len: usize)` | Returns an iterator over a range of packets with filters applied. |
| `Reader::seek(&mut self, packet: usize)` | Returns an iterator starting from the specified packet index. |
| `Reader::reload(&mut self)` | Reloads slot metadata and discovers packets appended after the reader was created. |

Filtering by blocks or payload improves performance by allowing the system to avoid fully parsing packets unless necessary.

### Storage Layout and Slot Design

The storage layer is based on how it organizes packets internally:
- Packets are not stored sequentially but are grouped into **slots**, with **500 packets per slot**.
- Each slot stores metadata about packet positions in the file and includes a **CRC** for slot validation, which makes the storage robust against corruption.
- Thanks to the slot metadata, `Reader` can **quickly locate packets by index** or **return a packet range efficiently**.

As previously mentioned, each slot maintains its own **CRC** to ensure data integrity. However, even if the storage file becomes corrupted and the slot metadata can no longer be trusted, packets remain accessible in a **manual recovery mode**. For example, you can use `PacketBufReader` to scan the file, ignore slot metadata, and extract intact packets sequentially.

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
