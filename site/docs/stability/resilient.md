## Resilient Compatibility

The `resilient` feature is a protocol-compatibility mode for `brec`.

Its purpose is simple:

- a newer writer may emit packets that contain blocks unknown to an older reader;
- an older reader should still be able to read the packet;
- unknown parts should be skipped deterministically instead of turning the whole packet into a hard failure.

In other words, `resilient` is the feature that makes forward-compatible packet evolution practical in `brec`.

## What The Feature Changes

When `resilient` is enabled:

- each block stores its body length right after the block signature;
- packet readers may skip unknown blocks by using that encoded length;
- unknown payloads may also be skipped, because payload headers already contain payload length;
- packet-level reads return both recognized data and metadata about skipped entities.

### Cost Of The Feature

`resilient` is not free, and it is better to say that directly.

The cost is small, but real:

- each block becomes larger by `4` bytes, because block body length is stored as `u32` right after the signature;
- block and packet readers perform additional validation checks around encoded lengths and packet boundaries;
- when an unknown entity is reported, its signature is copied into `Unrecognized`:
  - for blocks this is a fixed `[u8; 4]` copy;
  - for payloads this may involve copying the payload signature bytes into `Vec<u8>`.

In practice, this cost is not performance-critical.

For `brec`, the overhead here is on the scale of nanoseconds, and often fractions of a nanosecond per operation rather than anything operationally significant.

Still, it is important to understand the trade-off:

- you pay a few extra bytes per block;
- you pay a few extra checks on read;
- in return, you get forward-compatible packet evolution and deterministic skipping of unknown protocol parts.

The metadata is represented as:

```rust
enum UnrecognizedSignature {
    Block([u8; 4]),
    Payload(Vec<u8>),
}

struct Unrecognized {
    sig: UnrecognizedSignature,
    pos: Option<u64>,
    len: Option<u64>,
}
```

At packet level, successful reads become:

```rust
PacketReadStatus::Success((packet, skipped))
```

Where:

- `packet` contains all recognized blocks and the recognized payload, if any;
- `skipped` contains unknown blocks and/or payloads that were safely ignored.

## Typical Use Case

The main use case is rolling protocol evolution.

Example:

1. version 1 of a service knows blocks `A` and `B`;
2. version 2 adds a new block `C`;
3. version 2 writes packets containing `A + C + payload`;
4. version 1 reads the same packet;
5. version 1 does not know `C`, but still reads `A` and the payload successfully.

Without `resilient`, this packet would fail with `SignatureDismatch`.

With `resilient`, the unknown block is skipped and reported through `Vec<Unrecognized>`.

This is especially useful for:

- long-lived protocols that evolve gradually;
- mixed-version deployments during rolling upgrades;
- agents/clients that must tolerate newer indexing blocks;
- storage or stream processing pipelines where partial understanding is still valuable.

## Why This Matters In `brec`

`brec` packets are intentionally assembled from:

- a block set that acts as an indexing/filtering layer;
- an optional payload that carries the heavier data.

That architecture makes protocol evolution more flexible than in systems with a fixed packet catalog.

The `resilient` feature extends that idea:

- you can add new blocks without forcing every old reader to fail;
- old readers may continue using the blocks they know;
- payload handling may remain intact if the payload itself is still known.

This is why the "schema-free" claim in `brec` is not only about packet composition freedom, but also about practical compatibility during protocol growth.

## What Is Still A Hard Error

`resilient` is not a "best effort ignore everything" mode.

It only skips entities that are unknown by signature.

The following cases still fail hard:

- known block signature + invalid block parsing;
- known block signature + invalid block CRC;
- known payload signature + invalid payload parsing;
- known payload signature + invalid payload CRC;
- malformed encoded lengths;
- encoded lengths that exceed packet boundaries.

This is deliberate.

If a reader recognizes an entity, it is expected to validate it strictly.

## What `pos` And `len` Mean

For skipped entities:

- `pos` is the offset of the entity signature relative to the start of the packet;
- `len` is always the body length;
- for blocks, `len` does not include the block signature, the encoded `u32` length, or CRC;
- for payloads, `len` does not include the payload header.

This makes `Unrecognized` suitable for:

- logging;
- telemetry;
- compatibility diagnostics;
- protocol migration analysis.

## When You Should Use It

Use `resilient` when:

- you expect mixed protocol revisions in production;
- older consumers should survive newer indexing blocks;
- graceful forward compatibility is more important than strict binary lockstep;
- you want observability of skipped protocol parts instead of immediate failure.

## When You Should Not Use It

Do not enable `resilient` when:

- every reader and writer must use exactly the same protocol revision;
- strict format equality is part of your contract;
- any unknown protocol part must be treated as a fatal incompatibility;
- you want the smallest possible binary block layout and do not need forward-compatible block skipping.

Also note:

- protocols built with and without `resilient` are intentionally incompatible;
- `resilient` is a protocol choice, not a transparent runtime toggle.

If one side writes resilient blocks and the other side expects non-resilient blocks, parsing will fail.

## Practical Guidance

Use `resilient` primarily for protocol evolution at the block layer.

A good pattern is:

1. keep stable blocks for old readers;
2. add new optional blocks for newer readers;
3. let old readers skip what they do not understand;
4. monitor `Vec<Unrecognized>` during rollout.

This gives you a compatibility envelope without weakening validation of known data.
