pub mod json;
pub mod storage;
pub mod stream;
pub mod streamed_storage;
pub mod text;

use crate::{
    Level,
    test::{Packet, Payload},
};

pub fn is_match_payload(payload: &Payload) -> bool {
    match payload {
        Payload::String(msg) => msg.contains(crate::test::MATCH),
        Payload::AttachmentBincode(payload) => payload.name.contains(crate::test::MATCH),
        Payload::Bytes(_) => false,
    }
}

pub fn is_match_packet(packet: &Packet) -> bool {
    packet
        .blocks
        .iter()
        .map(|block| match block {
            crate::test::Block::Metadata(meta) => matches!(meta.level, Level::Err),
            _ => false,
        })
        .next()
        .unwrap_or(false)
        && packet.payload.as_ref().is_some_and(is_match_payload)
}
