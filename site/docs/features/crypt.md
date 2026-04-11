# Crypt Feature

`crypt` in `brec` adds transparent payload encryption and decryption.

The feature is payload-oriented:

- packet structure stays the same
- blocks are not encrypted
- only the payload body is encrypted
- encryption/decryption is driven through `PayloadContext`

This means the integration point is small: you keep using `Packet`, `PacketBufReader`, `Writer`, `Reader`, and the payload itself, but the payload is declared with crypto support and read/write operations receive crypto options.

Blocks staying visible is an intentional design decision, not a limitation.

That visibility is useful because blocks are often used for:

- packet filtering
- lightweight identification
- fast search and routing
- pre-decoding inspection

This preserves flexibility: sensitive payload data can be encrypted, while block-level metadata may still drive transport and indexing decisions.

Enabling the `crypt` feature also does not mean the whole protocol must become encrypted.

You can freely design a mixed protocol where:

- many packets stay open and readable
- only selected payloads are declared with `crypt`
- encrypted and non-encrypted packets coexist in the same application flow

That is usually the most reasonable approach: encrypt only the parts that actually carry sensitive data, and keep the rest simple and cheap to process.

## What the feature does

At the API level, `crypt` gives you:

- `EncryptOptions`
- `DecryptOptions`
- `CryptPolicy`
- `BricCryptCodec`
- crypto-aware payload generation through `#[payload(..., crypt)]`

Recommended path:

- use `#[payload(bincode, crypt)]` when possible

That is the most ergonomic path because serialization and deserialization are then provided out of the box, and `crypt` only adds transparent encryption/decryption around the encoded payload bytes.

At runtime, the current implementation uses:

- `ChaCha20Poly1305` for payload encryption
- RSA-OAEP-SHA256 for wrapping the session key
- an internal envelope that stores algorithm/version/session metadata

You usually do not work with the envelope directly. In the common case, `brec` handles it for you through payload encode/decode.

## Enable the feature

In `Cargo.toml`:

```toml
[dependencies]
brec = { path = "../brec", features = ["bincode", "crypt"] }
serde = { version = "1.0", features = ["derive"] }
```

If your payload uses `#[payload(bincode, crypt)]`, you need both:

- `bincode` for the payload serialization format
- `crypt` for encryption/decryption

With `bincode`, the payload gets automatic encode/decode support, so you do not have to manually implement the payload traits just to use encryption.

If you need the runtime-state side of this model first, see [Payload Context](../parts/context.md).

## Core idea

When a payload is declared as:

```rust
#[payload(bincode, crypt)]
pub struct MyPayload {
    pub message: String,
}
```

the generated crate-local `PayloadContext<'a>` gets crypto variants.

In practice, you pass one of these at the operation boundary:

```rust
let mut encrypt = PayloadContext::Encrypt(&mut encrypt_options);
let mut decrypt = PayloadContext::Decrypt(&mut decrypt_options);
```

So the mental model is simple:

- writer side uses `EncryptOptions`
- reader side uses `DecryptOptions`
- the payload itself stays regular Rust data

This does not create a conflict for mixed protocols.

- encrypted payloads read `PayloadContext::Encrypt(...)` / `PayloadContext::Decrypt(...)`
- non-encrypted payloads may coexist in the same generated payload family
- plain `#[payload(bincode)]` payloads simply do not use the crypto options stored in the context enum

## Minimal working example

This is the exact usage pattern validated in `examples/crypt`.

```rust
use brec::prelude::*;
use std::io::Cursor;

const PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
...
-----END PUBLIC KEY-----"#;

const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
...
-----END PRIVATE KEY-----"#;

const KEY_ID: &[u8] = b"demo-key";

#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    pub request_id: u32,
}

#[payload(bincode, crypt)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

brec::generate!();

fn usage() -> Result<GreetingPayload, Box<dyn std::error::Error>> {
    let original = GreetingPayload {
        message: "hello from encrypted payload".to_owned(),
    };

    let mut packet = Packet::new(
        vec![Block::MetaBlock(MetaBlock { request_id: 7 })],
        Some(Payload::GreetingPayload(original.clone())),
    );

    let mut encrypt = EncryptOptions::from_public_key_pem(PUBLIC_KEY_PEM)?
        .with_key_id(KEY_ID.to_vec());
    let mut encrypt_ctx = PayloadContext::Encrypt(&mut encrypt);

    let mut bytes = Vec::new();
    packet.write_all(&mut bytes, &mut encrypt_ctx)?;

    let mut decrypt = DecryptOptions::from_private_key_pem(PRIVATE_KEY_PEM)?
        .with_expected_key_id(KEY_ID.to_vec());
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);

    let mut source = Cursor::new(bytes.as_slice());
    let mut reader = PacketBufReader::new(&mut source);

    let packet = match reader.read(&mut decrypt_ctx)? {
        NextPacket::Found(packet) => packet,
        _ => return Err("packet was not restored".into()),
    };

    match packet.payload {
        Some(Payload::GreetingPayload(payload)) => Ok(payload),
        _ => Err("payload was not restored".into()),
    }
}
```

## How it plugs into payloads

`#[payload(bincode, crypt)]` means:

1. payload data is serialized with `bincode`
2. serialized payload bytes are encrypted before being written
3. encrypted bytes are decrypted before payload decoding

Important implications:

- you do not need to manually call `BricCryptCodec` for normal packet flow
- you do need to provide the right crypto context during read/write
- using the wrong context variant or missing crypto options will fail at runtime
- this requirement applies only to payloads declared with `crypt`; plain payloads can live in the same protocol and ignore the crypto context entirely

## Writer-side options

Use `EncryptOptions` on the encoding side.

Common constructors:

- `EncryptOptions::new(public_key)`
- `EncryptOptions::from_public_key_pem(...)`
- `EncryptOptions::from_public_key_pem_file(...)`
- `EncryptOptions::from_certificate_pem(...)`
- `EncryptOptions::from_certificate_pem_file(...)`
- `EncryptOptions::from_pem(...)`
- `EncryptOptions::from_pem_file(...)`

Common mutators:

- `with_key_id(...)`
- `clear_key_id()`
- `with_policy(...)`

Notes:

- the public key may come from raw public-key PEM or from an X509 certificate PEM
- `key_id` is optional, but strongly useful when multiple keys may exist at runtime
- `EncryptOptions` internally reuses parsed key material and caches session state

## Reader-side options

Use `DecryptOptions` on the decoding side.

Common constructors:

- `DecryptOptions::new(private_key)`
- `DecryptOptions::from_private_key_pem(...)`
- `DecryptOptions::from_private_key_pem_file(...)`
- `DecryptOptions::from_pem(...)`
- `DecryptOptions::from_pem_file(...)`

Common mutators:

- `with_expected_key_id(...)`
- `clear_expected_key_id()`
- `with_policy(...)`

Notes:

- decryption requires the RSA private key
- if `with_expected_key_id(...)` is set, envelopes without matching `key_id` will be rejected
- `DecryptOptions` caches unwrapped session keys for repeated use

## `key_id` behavior

`key_id` is optional metadata embedded into the crypto envelope.

Recommended pattern:

- writer sets `EncryptOptions::with_key_id(...)`
- reader sets `DecryptOptions::with_expected_key_id(...)`

This gives you a cheap guard against decrypting with the wrong logical key configuration.

If the reader expects `key_id` and the envelope:

- has no `key_id`, you get `MissingKeyId`
- has a different `key_id`, you get `KeyIdMismatch`

## `CryptPolicy`

`CryptPolicy` controls runtime caching behavior:

```rust
pub struct CryptPolicy {
    pub session_reuse_limit: u32,
    pub decrypt_cache_limit: usize,
}
```

Default values:

- `session_reuse_limit = 100`
- `decrypt_cache_limit = 32`

What they mean:

- `session_reuse_limit`: how many payloads may reuse the same encryption session before a new one is created
- `decrypt_cache_limit`: how many decrypted session entries are kept on the reader side

Why these settings exist:

- wrapping and unwrapping a fresh RSA session key for every single message is expensive
- on high-throughput streams with many small messages, doing that work every time would noticeably reduce throughput
- bounded reuse and bounded decrypt-side caching are a practical compromise between performance and crypto session churn

In practice, values in the `50..100` range are often a sensible starting point for busy message streams when you want to keep throughput stable without letting reuse grow unbounded.

Typical use:

```rust
let policy = CryptPolicy {
    session_reuse_limit: 200,
    decrypt_cache_limit: 64,
};

let encrypt = EncryptOptions::from_public_key_pem(public_pem)?.with_policy(policy);
let decrypt = DecryptOptions::from_private_key_pem(private_pem)?.with_policy(policy);
```

## When to use `BricCryptCodec` directly

Most users should not call `BricCryptCodec` directly during normal packet I/O.

It is useful when you need payload-level crypto outside packet read/write, for example:

- encrypt raw payload bytes manually
- decrypt previously stored encrypted payload bytes
- inspect or parse the internal crypto envelope

Useful methods:

- `BricCryptCodec::encrypt(...)`
- `BricCryptCodec::decrypt(...)`
- `BricCryptCodec::encrypt_payload(...)`
- `BricCryptCodec::decrypt_payload(...)`
- `BricCryptCodec::parse(...)`
- `BricCryptCodec::format(...)`

## Error model

Crypto errors are represented by `CryptError` and convert to `std::io::Error`.

Typical categories:

- invalid key material: `InvalidRsaPublicKeyPem`, `InvalidRsaPrivateKeyPem`
- envelope metadata mismatch: `MissingKeyId`, `KeyIdMismatch`
- cryptographic failure: `EncryptPayloadBody`, `DecryptPayloadBody`, `WrapSessionKey`, `UnwrapSessionKey`
- format mismatch: `UnsupportedEnvelopeVersion`, `UnsupportedAlgorithm`, `MalformedEnvelope`

In normal packet flow these surface as I/O errors, so packet readers and writers stay compatible with the rest of the library API.

## Practical rules

- Encrypt only payloads that actually require confidentiality.
- Keep blocks non-sensitive, because blocks remain visible for packet scanning/filtering.
- Reuse `EncryptOptions` and `DecryptOptions` instances when processing many packets; the feature is designed for that.
- Prefer setting `key_id` whenever the application may rotate or select keys dynamically.
- Keep the PEM loading boundary outside hot loops when possible.

## Reference points in this repository

- example usage: `examples/crypt`
- stress coverage: `tests/stress_payloads_crypt`

If you need a valid starting point, `examples/crypt/src/main.rs` is the canonical minimal example for this feature in the repository.
