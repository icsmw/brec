use brec::prelude::*;

#[cfg(test)]
const MAX_PAYLOAD_LEN: u32 = 7;
#[cfg(test)]
const MAX_PACKET_LEN: u64 = 31;
#[cfg(test)]
const INITIAL_PACKET_BUFFER_CAPACITY: usize = 5;

#[block]
#[derive(Debug)]
pub struct TestBlock {
    value: u8,
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct TestPayload {
    value: u8,
}

brec::generate!(
    default_max_payload_len = 7,
    default_max_packet_len = 31,
    default_initial_packet_buffer_capacity = 5,
);

#[cfg(test)]
fn packet_header(size: u64, blocks_len: u64, payload: bool) -> PacketHeader {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&size.to_le_bytes());
    hasher.update(&blocks_len.to_le_bytes());
    hasher.update(if payload { &[1] } else { &[0] });
    PacketHeader {
        size,
        blocks_len,
        payload,
        crc: hasher.finalize(),
    }
}

#[test]
fn custom_scheme_payload() {
    assert_eq!(Payload::MAX_PAYLOAD_LEN, MAX_PAYLOAD_LEN);
    assert_eq!(Payload::MAX_PACKET_LEN, MAX_PACKET_LEN);
    assert_eq!(
        Payload::INITIAL_PACKET_BUFFER_CAPACITY,
        INITIAL_PACKET_BUFFER_CAPACITY
    );
    assert_eq!(TestPayload::MAX_PAYLOAD_LEN, MAX_PAYLOAD_LEN);
    assert_eq!(TestPayload::MAX_PACKET_LEN, MAX_PACKET_LEN);

    let payload_header = PayloadHeader {
        sig: ByteBlock::Len4(*b"test"),
        crc: ByteBlock::Len4([0; 4]),
        len: MAX_PAYLOAD_LEN + 1,
    };
    let mut input = std::io::Cursor::new(payload_header.as_vec());
    assert!(matches!(
        PayloadHeader::read::<_, Payload>(&mut input),
        Err(Error::InvalidLength)
    ));

    let packet_header = packet_header(MAX_PACKET_LEN + 1, 0, false);
    let mut input = Vec::new();
    packet_header.write_all(&mut input).unwrap();
    let mut input = std::io::Cursor::new(input);
    assert!(matches!(
        PacketHeader::read::<_, Payload>(&mut input),
        Err(Error::InvalidLength)
    ));
}

fn main() {}
