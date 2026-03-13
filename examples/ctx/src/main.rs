use std::io::{Error, ErrorKind};

use brec::prelude::*;

/// A small example block attached to every packet.
///
/// Blocks are useful for cheap prefiltering: the reader can inspect them before
/// decoding the full payload and, when possible, without extra allocations.
/// A packet may contain up to 255 blocks.
#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    pub request_id: u32,
}

/// User-defined payload context.
///
/// Context is passed to payload encode/decode operations in mutable form, which
/// makes it suitable for runtime state such as prefixes, caches, crypto keys,
/// or other temporary data required only during processing.
///
/// The `#[payload(ctx)]` attribute tells `brec::generate!()` to add a matching
/// variant to the generated `PayloadContext<'a>` enum:
/// `PayloadContext::PrefixContext(&'a mut PrefixContext)`.
#[payload(ctx)]
pub struct PrefixContext {
    pub prefix: String,
}

impl PrefixContext {
    /// Extracts the expected context variant for `GreetingPayload`.
    fn extract_prefix<'a>(ctx: &'a mut crate::PayloadContext<'_>) -> std::io::Result<&'a str> {
        match ctx {
            crate::PayloadContext::PrefixContext(options) => Ok(options.prefix.as_str()),
            crate::PayloadContext::None => Err(Error::new(
                ErrorKind::InvalidInput,
                "GreetingPayload expects PayloadContext::PrefixContext",
            )),
        }
    }
}

/// Packet payload used in this example.
///
/// The payload itself stores only the logical message. The prefix is supplied
/// externally through `PrefixContext` and is injected during encoding.
///
/// If you do not need manual payload trait implementations, see
/// `examples/bincode`: there the same packet flow is shown with
/// `#[payload(bincode)]`, where `serde` is enough and the manual work below is
/// not needed.
#[payload]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

/// Manual payload encoding.
///
/// We do not use `#[payload(bincode)]` here. Instead, the example shows that
/// you can implement payload traits yourself and still use any serializer you
/// prefer internally. If you want to avoid this manual work, see
/// `examples/bincode`.
impl PayloadEncode for GreetingPayload {
    fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
        // 1. Extract the user-defined runtime context.
        let prefix = PrefixContext::extract_prefix(ctx)?;

        // 2. Build the on-wire representation from logical payload data.
        let mut wire = self.clone();
        wire.message = format!("{prefix}{}", self.message);

        // 3. Serialize it into bytes.
        bincode::serde::encode_to_vec(&wire, bincode::config::standard())
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}

/// Optional borrowed encoding fast path.
///
/// Payloads may return a borrowed byte slice when they already keep an encoded
/// representation internally. This can avoid extra allocations and reduce I/O.
/// In this simple example we always build a fresh buffer, so the fast path is
/// not available.
impl PayloadEncodeReferred for GreetingPayload {
    fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
        Ok(None)
    }
}

/// Manual payload decoding.
///
/// The decode side receives both the raw bytes and the same user-defined
/// context. Here we use the context to validate and remove the prefix that was
/// added during encoding.
impl PayloadDecode<GreetingPayload> for GreetingPayload {
    fn decode(buf: &[u8], ctx: &mut Self::Context<'_>) -> std::io::Result<GreetingPayload> {
        // 1. Read the prefix from the runtime context.
        let prefix = PrefixContext::extract_prefix(ctx)?.to_owned();

        // 2. Deserialize the wire representation.
        let (mut payload, _): (GreetingPayload, usize) =
            bincode::serde::decode_from_slice(buf, bincode::config::standard())
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;

        // 3. Convert wire data back to logical payload data.
        payload.message = payload
            .message
            .strip_prefix(&prefix)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing context prefix"))?
            .to_owned();

        Ok(payload)
    }
}

/// Payload size calculation.
///
/// `brec` needs to know the encoded payload size when writing packet headers.
/// Here we simply reuse the manual encoder.
impl PayloadSize for GreetingPayload {
    fn size(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<u64> {
        Ok(PayloadEncode::encode(self, ctx)?.len() as u64)
    }
}

/// Default payload CRC implementation.
///
/// By default `brec` computes CRC32 over the encoded payload bytes.
impl PayloadCrc for GreetingPayload {}

// Generates crate-local glue code:
// - `Block`
// - `Payload`
// - `Packet`
// - `PayloadContext<'a>`
// - `PacketBufReader`
// - `Reader` / `Writer`
brec::generate!();

fn main() {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn context_aware_packet_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original = GreetingPayload {
            message: "hello from payload context".to_owned(),
        };
        let mut packet = Packet::new(
            vec![Block::MetaBlock(MetaBlock { request_id: 7 })],
            Some(Payload::GreetingPayload(original.clone())),
        );

        let mut encode_options = PrefixContext {
            prefix: "[ctx] ".to_owned(),
        };
        let mut encode_ctx = PayloadContext::PrefixContext(&mut encode_options);

        let mut bytes = Vec::new();
        packet.write_all(&mut bytes, &mut encode_ctx)?;

        let mut source = Cursor::new(bytes);
        let mut reader = PacketBufReader::new(&mut source);

        let mut decode_options = PrefixContext {
            prefix: "[ctx] ".to_owned(),
        };
        let mut decode_ctx = PayloadContext::PrefixContext(&mut decode_options);

        let packet = match reader.read(&mut decode_ctx)? {
            NextPacket::Found(packet) => packet,
            NextPacket::NotEnoughData(_) => {
                return Err("unexpected read status: NotEnoughData".into());
            }
            NextPacket::NoData => return Err("unexpected read status: NoData".into()),
            NextPacket::NotFound => return Err("unexpected read status: NotFound".into()),
            NextPacket::Skipped => return Err("unexpected read status: Skipped".into()),
        };

        let restored = match packet.payload {
            Some(Payload::GreetingPayload(payload)) => payload,
            _ => return Err("payload was not restored".into()),
        };

        assert_eq!(restored, original);
        Ok(())
    }
}
