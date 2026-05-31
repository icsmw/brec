use std::io::Cursor;

use brec::prelude::*;

#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    pub request_id: u32,
}

#[payload(bincode)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

brec::generate!();

#[derive(Default, Debug, PartialEq, Eq)]
struct ReaderStats {
    ignored_bytes: usize,
    found_packets: usize,
    not_found_results: usize,
    no_data_results: usize,
}

fn packet_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut packet = Packet::new(
        vec![Block::MetaBlock(MetaBlock { request_id: 42 })],
        Some(Payload::GreetingPayload(GreetingPayload {
            message: "hello from workflow context".to_owned(),
        })),
    );

    let mut bytes = Vec::new();
    packet.write_all(&mut bytes, &mut brec::default_payload_context())?;
    Ok(bytes)
}

fn decode_one_packet_with_stats(
    bytes: Vec<u8>,
) -> Result<(Packet, ReaderStats), Box<dyn std::error::Error>> {
    let mut source = Cursor::new(bytes);
    let mut reader = PacketBufReader::with_context(&mut source, ReaderStats::default());

    reader.add_rule(Rule::IgnoredControl(RuleFnDef::Static(|ignored, stats| {
        stats.ignored_bytes += ignored.len();
        Ok(IgnoredAction::Continue)
    })))?;

    reader.add_rule(Rule::NextPacket(RuleFnDef::Static(|next, stats| {
        match next {
            NextPacket::Found(_) => stats.found_packets += 1,
            NextPacket::NotFound => stats.not_found_results += 1,
            NextPacket::NoData => stats.no_data_results += 1,
            NextPacket::NotEnoughData(_) | NextPacket::Skipped => {}
        }
        Ok(())
    })))?;

    let mut ctx = brec::default_payload_context();
    let packet = match reader.read(&mut ctx)? {
        NextPacket::Found(packet) => packet,
        NextPacket::NotEnoughData(_) => {
            return Err("unexpected read status: NotEnoughData".into());
        }
        NextPacket::NoData => return Err("unexpected read status: NoData".into()),
        NextPacket::NotFound => return Err("unexpected read status: NotFound".into()),
        NextPacket::Skipped => return Err("unexpected read status: Skipped".into()),
    };

    match reader.read(&mut ctx)? {
        NextPacket::NoData => {}
        NextPacket::Found(_) => return Err("unexpected second packet".into()),
        NextPacket::NotEnoughData(_) => {
            return Err("unexpected read status: NotEnoughData".into());
        }
        NextPacket::NotFound => return Err("unexpected read status: NotFound".into()),
        NextPacket::Skipped => return Err("unexpected read status: Skipped".into()),
    }

    Ok((packet, reader.into_context()))
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_context_collects_reader_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let junk_prefix = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut stream = junk_prefix.to_vec();
        stream.extend_from_slice(&packet_bytes()?);

        let (packet, stats) = decode_one_packet_with_stats(stream)?;

        assert_eq!(
            packet.blocks,
            vec![Block::MetaBlock(MetaBlock { request_id: 42 })]
        );
        assert_eq!(
            packet.payload,
            Some(Payload::GreetingPayload(GreetingPayload {
                message: "hello from workflow context".to_owned(),
            }))
        );

        assert_eq!(
            stats,
            ReaderStats {
                ignored_bytes: junk_prefix.len(),
                found_packets: 1,
                not_found_results: 0,
                no_data_results: 1,
            }
        );

        Ok(())
    }
}
