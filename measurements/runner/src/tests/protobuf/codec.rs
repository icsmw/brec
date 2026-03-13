use super::model::{
    PbAttachment, PbBlockBorrowed, PbMetadata, PbPacket, PbRecord, PbRecordBinary, PbRecordBorrowed,
    PbRecordCrypt, invalid_data,
};
use crate::test::{Block, MATCH, Payload, WrappedPacket};
use crate::*;
use brec::prelude::{DecryptOptions, EncryptOptions};
use prost::Message;

fn metadata_from_packet(packet: &WrappedPacket) -> std::io::Result<PbMetadata> {
    let meta = packet
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Metadata(meta) => Some(meta),
            _ => None,
        })
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing metadata"))?;

    Ok(PbMetadata {
        level: u8::from(&meta.level) as u32,
        target: u8::from(&meta.target) as u32,
        tm: meta.tm,
    })
}

fn payload_from_packet<'a>(packet: &'a WrappedPacket) -> std::io::Result<&'a Payload> {
    packet
        .payload
        .as_ref()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing payload"))
}

fn borrowed_from_packet(packet: &WrappedPacket) -> std::io::Result<PbBlockBorrowed> {
    let block = packet
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::BlockBorrowed(block) => Some(block),
            _ => None,
        })
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing borrowed block"))?;

    Ok(PbBlockBorrowed {
        field_u8: block.field_u8 as u32,
        field_u16: block.field_u16 as u32,
        field_u32: block.field_u32,
        field_u64: block.field_u64,
        field_u128: block.field_u128.to_le_bytes().to_vec(),
        field_i8: block.field_i8 as i32,
        field_i16: block.field_i16 as i32,
        field_i32: block.field_i32,
        field_i64: block.field_i64,
        field_i128: block.field_i128.to_le_bytes().to_vec(),
        field_f32: block.field_f32,
        field_f64: block.field_f64,
        field_bool: block.field_bool,
        blob_a: block.blob_a.to_vec(),
        blob_b: block.blob_b.to_vec(),
    })
}

fn attachment_from_payload(payload: &Payload) -> std::io::Result<PbAttachment> {
    match payload {
        Payload::AttachmentBincode(atc) => Ok(PbAttachment {
            uuid: atc.uuid.clone(),
            name: atc.name.clone(),
            chunk: atc.chunk,
            data: atc.data.clone(),
            fields: atc.fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }),
        Payload::AttachmentCrypt(atc) => Ok(PbAttachment {
            uuid: atc.uuid.clone(),
            name: atc.name.clone(),
            chunk: atc.chunk,
            data: atc.data.clone(),
            fields: atc.fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unexpected payload type",
        )),
    }
}

fn encrypted_payload_bytes(payload: &Payload, encrypt: &mut EncryptOptions) -> std::io::Result<Vec<u8>> {
    let atc = attachment_from_payload(payload)?;
    let mut encoded_payload = Vec::new();
    atc.encode(&mut encoded_payload).map_err(invalid_data)?;
    tests::crypt::encrypt_bytes(&encoded_payload, encrypt)
}

pub(crate) fn packet_to_pb(
    payload_kind: report::PayloadKind,
    packet: &WrappedPacket,
    encrypt: Option<&mut EncryptOptions>,
) -> std::io::Result<PbPacket> {
    match payload_kind {
        report::PayloadKind::Record => {
            let msg = match payload_from_packet(packet)? {
                Payload::String(msg) => msg.clone(),
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "expected text payload",
                    ));
                }
            };
            Ok(PbPacket::Record(PbRecord {
                meta: Some(metadata_from_packet(packet)?),
                msg,
            }))
        }
        report::PayloadKind::RecordBincode => Ok(PbPacket::RecordBinary(PbRecordBinary {
            meta: Some(metadata_from_packet(packet)?),
            payload: Some(attachment_from_payload(payload_from_packet(packet)?)?),
        })),
        report::PayloadKind::RecordCrypt => {
            let enc = encrypt.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "encrypt options required for RecordCrypt",
                )
            })?;
            Ok(PbPacket::RecordCrypt(PbRecordCrypt {
                meta: Some(metadata_from_packet(packet)?),
                payload_encrypted: encrypted_payload_bytes(payload_from_packet(packet)?, enc)?,
            }))
        }
        report::PayloadKind::Borrowed => Ok(PbPacket::Borrowed(PbRecordBorrowed {
            block: Some(borrowed_from_packet(packet)?),
        })),
    }
}

pub(crate) fn pb_to_bytes(packet: &PbPacket) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    match packet {
        PbPacket::Record(row) => row.encode(&mut bytes).map_err(invalid_data)?,
        PbPacket::RecordBinary(row) => row.encode(&mut bytes).map_err(invalid_data)?,
        PbPacket::RecordCrypt(row) => row.encode(&mut bytes).map_err(invalid_data)?,
        PbPacket::Borrowed(row) => row.encode(&mut bytes).map_err(invalid_data)?,
    }
    Ok(bytes)
}

pub(crate) fn decode_for_read(
    payload_kind: report::PayloadKind,
    data: &[u8],
    decrypt: &mut DecryptOptions,
) -> std::io::Result<()> {
    match payload_kind {
        report::PayloadKind::Record => {
            let _ = PbRecord::decode(data).map_err(invalid_data)?;
        }
        report::PayloadKind::RecordBincode => {
            let _ = PbRecordBinary::decode(data).map_err(invalid_data)?;
        }
        report::PayloadKind::RecordCrypt => {
            let row = PbRecordCrypt::decode(data).map_err(invalid_data)?;
            let decrypted_payload = tests::crypt::decrypt_bytes(&row.payload_encrypted, decrypt)?;
            let _ = PbAttachment::decode(decrypted_payload.as_slice()).map_err(invalid_data)?;
        }
        report::PayloadKind::Borrowed => {
            let _ = PbRecordBorrowed::decode(data).map_err(invalid_data)?;
        }
    }
    Ok(())
}

pub(crate) fn decode_borrowed_for_read(data: &[u8]) -> std::io::Result<()> {
    let _ = PbRecordBorrowed::decode(data).map_err(invalid_data)?;
    Ok(())
}

fn metadata_match(meta: &PbMetadata) -> bool {
    meta.level == u8::from(&Level::Err) as u32
}

fn attachment_match(payload: &PbAttachment) -> bool {
    payload.name.contains(MATCH)
}

pub(crate) fn record_matches(
    payload_kind: report::PayloadKind,
    data: &[u8],
    decrypt: Option<&mut DecryptOptions>,
) -> std::io::Result<bool> {
    match payload_kind {
        report::PayloadKind::Record => {
            let row = PbRecord::decode(data).map_err(invalid_data)?;
            Ok(row.meta.as_ref().is_some_and(metadata_match) && row.msg.contains(MATCH))
        }
        report::PayloadKind::RecordBincode => {
            let row = PbRecordBinary::decode(data).map_err(invalid_data)?;
            Ok(row.meta.as_ref().is_some_and(metadata_match)
                && row.payload.as_ref().is_some_and(attachment_match))
        }
        report::PayloadKind::RecordCrypt => {
            let row = PbRecordCrypt::decode(data).map_err(invalid_data)?;
            let dec = decrypt.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "decrypt options required for RecordCrypt",
                )
            })?;
            let decrypted_payload = tests::crypt::decrypt_bytes(&row.payload_encrypted, dec)?;
            let payload = PbAttachment::decode(decrypted_payload.as_slice()).map_err(invalid_data)?;
            Ok(row.meta.as_ref().is_some_and(metadata_match) && attachment_match(&payload))
        }
        report::PayloadKind::Borrowed => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Borrowed payload is not supported in filtering scenario",
        )),
    }
}
