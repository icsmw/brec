use std::io::Cursor;

use brec::prelude::*;

/// The simplest possible block: it just tags the packet with a request id.
/// Handy when the reader wants to prefilter packets before touching payload bytes.
#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    pub request_id: u32,
}

/// A second block, just to keep the example a bit more interesting.
/// Here it represents packet priority.
#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityBlock {
    pub level: u8,
}

/// A compact text payload.
/// Useful for commands, status messages, or any other human-readable records.
#[payload(bincode)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

/// Another payload variant, this time more application-like.
/// It carries raw bytes, a small flag, and a response code.
#[payload(bincode)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BinaryPayload {
    pub tag: String,
    pub compressed: bool,
    pub bytes: Vec<u8>,
    pub status: u16,
}

brec::generate!();

#[derive(Clone)]
struct PacketExample {
    name: &'static str,
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

impl PacketExample {
    fn packet(&self) -> Packet {
        Packet::new(self.blocks.clone(), self.payload.clone())
    }
}

fn example_packets() -> Vec<PacketExample> {
    vec![
        PacketExample {
            name: "payload only",
            // The minimal packet shape: payload only, no blocks at all.
            blocks: vec![],
            payload: Some(Payload::GreetingPayload(GreetingPayload {
                message: "hello from a packet without blocks".to_owned(),
            })),
        },
        PacketExample {
            name: "blocks only",
            // The opposite case: blocks are present, but there is no payload.
            // That can be enough when the packet itself already acts as a signal.
            blocks: vec![
                Block::MetaBlock(MetaBlock { request_id: 7 }),
                Block::PriorityBlock(PriorityBlock { level: 2 }),
            ],
            payload: None,
        },
        PacketExample {
            name: "two blocks and greeting payload",
            // A more complete packet: metadata in blocks and actual data in payload.
            blocks: vec![
                Block::MetaBlock(MetaBlock { request_id: 8 }),
                Block::PriorityBlock(PriorityBlock { level: 9 }),
            ],
            payload: Some(Payload::GreetingPayload(GreetingPayload {
                message: "this one carries both blocks and text".to_owned(),
            })),
        },
        PacketExample {
            name: "binary payload with one block",
            blocks: vec![Block::MetaBlock(MetaBlock { request_id: 9 })],
            payload: Some(Payload::BinaryPayload(BinaryPayload {
                tag: "snapshot".to_owned(),
                compressed: true,
                bytes: vec![0xCA, 0xFE, 0xBA, 0xBE],
                status: 206,
            })),
        },
    ]
}

fn describe_blocks(blocks: &[Block]) -> String {
    if blocks.is_empty() {
        return "no blocks".to_owned();
    }

    blocks
        .iter()
        .map(|block| match block {
            Block::MetaBlock(block) => format!("MetaBlock(request_id={})", block.request_id),
            Block::PriorityBlock(block) => format!("PriorityBlock(level={})", block.level),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn describe_payload(payload: &Option<Payload>) -> String {
    match payload {
        None => "no payload".to_owned(),
        Some(Payload::GreetingPayload(payload)) => {
            format!("GreetingPayload(message={:?})", payload.message)
        }
        Some(Payload::BinaryPayload(payload)) => format!(
            "BinaryPayload(tag={:?}, compressed={}, bytes={}, status={})",
            payload.tag,
            payload.compressed,
            payload.bytes.len(),
            payload.status
        ),
        Some(Payload::String(payload)) => format!("String({payload:?})"),
        Some(Payload::Bytes(payload)) => format!("Bytes(len={})", payload.len()),
    }
}

fn encode_packets(examples: &[PacketExample]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    let mut ctx = brec::default_payload_context();

    for example in examples {
        let mut packet = example.packet();
        packet.write_all(&mut bytes, &mut ctx)?;
    }

    Ok(bytes)
}

fn decode_packets(bytes: Vec<u8>) -> Result<Vec<Packet>, Box<dyn std::error::Error>> {
    let mut source = Cursor::new(bytes);
    let mut reader = PacketBufReader::new(&mut source);
    let mut ctx = brec::default_payload_context();
    let mut packets = Vec::new();

    loop {
        match reader.read(&mut ctx)? {
            NextPacket::Found(packet) => packets.push(packet),
            NextPacket::NoData => break,
            NextPacket::NotEnoughData(_) => {
                return Err("unexpected read status: NotEnoughData".into());
            }
            NextPacket::NotFound => return Err("unexpected read status: NotFound".into()),
            NextPacket::Skipped => return Err("unexpected read status: Skipped".into()),
        }
    }

    Ok(packets)
}

fn collect_showcase_lines() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let examples = example_packets();
    let mut lines = Vec::new();

    lines.push(format!("Encoding {} packets...", examples.len()));
    for example in &examples {
        lines.push(format!(
            "- {} -> {}, {}",
            example.name,
            describe_blocks(&example.blocks),
            describe_payload(&example.payload),
        ));
    }

    let bytes = encode_packets(&examples)?;
    lines.push(format!("Serialized stream size: {} bytes", bytes.len()));

    let restored = decode_packets(bytes)?;
    lines.push(format!(
        "Decoded {} packets back from the stream.",
        restored.len()
    ));

    for (index, packet) in restored.iter().enumerate() {
        lines.push(format!(
            "  [{}] {}, {}",
            index,
            describe_blocks(&packet.blocks),
            describe_payload(&packet.payload),
        ));
    }

    Ok(lines)
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_examples_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let examples = example_packets();
        let bytes = encode_packets(&examples)?;
        let restored = decode_packets(bytes)?;

        assert_eq!(restored.len(), examples.len());

        for (expected, actual) in examples.iter().zip(restored.iter()) {
            assert_eq!(actual.blocks, expected.blocks, "block mismatch");
            assert_eq!(actual.payload, expected.payload, "payload mismatch");
        }

        Ok(())
    }

    #[test]
    fn packet_examples_showcase() -> Result<(), Box<dyn std::error::Error>> {
        for line in collect_showcase_lines()? {
            println!("{line}");
        }

        Ok(())
    }
}
