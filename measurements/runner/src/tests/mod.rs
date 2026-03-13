pub mod crypt;
pub mod flatbuffers;
pub mod json;
pub mod protobuf;
pub mod storage;
pub mod stream;
pub mod text;

use crate::{
    Level,
    test::{Packet, Payload},
};

pub fn is_match_payload(payload: &Payload) -> bool {
    match payload {
        Payload::String(msg) => msg.contains(crate::test::MATCH),
        Payload::AttachmentBincode(payload) => payload.name.contains(crate::test::MATCH),
        Payload::AttachmentCrypt(payload) => payload.name.contains(crate::test::MATCH),
        Payload::Bytes(_) => false,
    }
}

pub fn is_match_packet(packet: &Packet) -> bool {
    packet
        .blocks
        .iter()
        .map(|block| matches!(block, crate::test::Block::Metadata(meta) if matches!(meta.level, Level::Err)))
        .next()
        .unwrap_or(false)
        && packet.payload.as_ref().is_some_and(is_match_payload)
}
