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

/// Packet payload used in this example.
///
/// This example uses `#[payload(bincode)]`, so no manual payload trait
/// implementations are required. It is the "zero manual work" counterpart to
/// `examples/ctx`, where the same ideas are shown with full manual control.
///
/// If `serde` is enough for your payload, prefer this approach.
#[payload(bincode)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

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
    fn bincode_payload_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original = GreetingPayload {
            message: "hello from bincode payload".to_owned(),
        };
        let mut packet = Packet::new(
            vec![Block::MetaBlock(MetaBlock { request_id: 7 })],
            Some(Payload::GreetingPayload(original.clone())),
        );

        let mut bytes = Vec::new();
        let mut encode_ctx = brec::default_payload_context();
        packet.write_all(&mut bytes, &mut encode_ctx)?;

        let mut source = Cursor::new(bytes);
        let mut reader = PacketBufReader::new(&mut source);

        let mut decode_ctx = brec::default_payload_context();

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
