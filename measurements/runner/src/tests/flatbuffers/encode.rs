use super::common::{
    FbTableOffset, VT_ATTACHMENT_CHUNK, VT_ATTACHMENT_DATA, VT_ATTACHMENT_FIELDS, VT_ATTACHMENT_NAME,
    VT_ATTACHMENT_UUID, VT_BORROWED_BLOB_A, VT_BORROWED_BLOB_B, VT_BORROWED_FIELD_BOOL,
    VT_BORROWED_FIELD_F32, VT_BORROWED_FIELD_F64, VT_BORROWED_FIELD_I8, VT_BORROWED_FIELD_I16,
    VT_BORROWED_FIELD_I32, VT_BORROWED_FIELD_I64, VT_BORROWED_FIELD_I128, VT_BORROWED_FIELD_U8,
    VT_BORROWED_FIELD_U16, VT_BORROWED_FIELD_U32, VT_BORROWED_FIELD_U64, VT_BORROWED_FIELD_U128,
    VT_FIELD_KEY, VT_FIELD_VALUE, VT_META_LEVEL, VT_META_TARGET, VT_META_TM, VT_RECORD_BINARY_META,
    VT_RECORD_BINARY_PAYLOAD, VT_RECORD_BORROWED_BLOCK, VT_RECORD_CRYPT_META,
    VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED, VT_RECORD_META, VT_RECORD_MSG, invalid_data,
};
use crate::test::{Block, Payload, WrappedPacket};
use crate::*;
use brec::prelude::EncryptOptions;
use flatbuffers::FlatBufferBuilder;

fn metadata_from_packet(packet: &WrappedPacket) -> std::io::Result<&Metadata> {
    packet
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Metadata(meta) => Some(meta),
            _ => None,
        })
        .ok_or_else(|| invalid_data("missing metadata"))
}

fn payload_from_packet(packet: &WrappedPacket) -> std::io::Result<&Payload> {
    packet
        .payload
        .as_ref()
        .ok_or_else(|| invalid_data("missing payload"))
}

fn borrowed_block_from_packet(packet: &WrappedPacket) -> std::io::Result<&BlockBorrowed> {
    packet
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::BlockBorrowed(block) => Some(block),
            _ => None,
        })
        .ok_or_else(|| invalid_data("missing borrowed block"))
}

fn build_metadata_table(builder: &mut FlatBufferBuilder<'_>, meta: &Metadata) -> FbTableOffset {
    let start = builder.start_table();
    builder.push_slot::<u8>(VT_META_LEVEL, u8::from(&meta.level), 0);
    builder.push_slot::<u8>(VT_META_TARGET, u8::from(&meta.target), 0);
    builder.push_slot::<u64>(VT_META_TM, meta.tm, 0);
    builder.end_table(start)
}

fn build_attachment_table_from_payload(
    builder: &mut FlatBufferBuilder<'_>,
    payload: &Payload,
) -> std::io::Result<FbTableOffset> {
    let (uuid, name, chunk, data, fields) = match payload {
        Payload::AttachmentBincode(atc) => (&atc.uuid, &atc.name, atc.chunk, &atc.data, &atc.fields),
        Payload::AttachmentCrypt(atc) => (&atc.uuid, &atc.name, atc.chunk, &atc.data, &atc.fields),
        _ => return Err(invalid_data("unexpected payload type")),
    };

    let uuid_off = builder.create_string(uuid.as_str());
    let name_off = builder.create_string(name.as_str());
    let data_off = builder.create_vector(data.as_slice());

    let mut field_offsets = Vec::with_capacity(fields.len());
    for (key, value) in fields.iter() {
        let key_off = builder.create_string(key.as_str());
        let value_off = builder.create_string(value.as_str());
        let field_start = builder.start_table();
        builder.push_slot_always(VT_FIELD_KEY, key_off);
        builder.push_slot_always(VT_FIELD_VALUE, value_off);
        field_offsets.push(builder.end_table(field_start));
    }
    let fields_off = builder.create_vector(field_offsets.as_slice());

    let start = builder.start_table();
    builder.push_slot_always(VT_ATTACHMENT_UUID, uuid_off);
    builder.push_slot_always(VT_ATTACHMENT_NAME, name_off);
    builder.push_slot::<u32>(VT_ATTACHMENT_CHUNK, chunk, 0);
    builder.push_slot_always(VT_ATTACHMENT_DATA, data_off);
    builder.push_slot_always(VT_ATTACHMENT_FIELDS, fields_off);
    Ok(builder.end_table(start))
}

fn build_attachment_bytes(payload: &Payload) -> std::io::Result<Vec<u8>> {
    let mut builder = FlatBufferBuilder::new();
    let root = build_attachment_table_from_payload(&mut builder, payload)?;
    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

fn build_record_bytes(meta: &Metadata, payload: &Payload) -> std::io::Result<Vec<u8>> {
    let msg = match payload {
        Payload::String(msg) => msg.as_str(),
        _ => return Err(invalid_data("expected text payload")),
    };
    let mut builder = FlatBufferBuilder::new();
    let meta_off = build_metadata_table(&mut builder, meta);
    let msg_off = builder.create_string(msg);
    let start = builder.start_table();
    builder.push_slot_always(VT_RECORD_META, meta_off);
    builder.push_slot_always(VT_RECORD_MSG, msg_off);
    let root = builder.end_table(start);
    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

fn build_record_binary_bytes(meta: &Metadata, payload: &Payload) -> std::io::Result<Vec<u8>> {
    let mut builder = FlatBufferBuilder::new();
    let meta_off = build_metadata_table(&mut builder, meta);
    let payload_off = build_attachment_table_from_payload(&mut builder, payload)?;
    let start = builder.start_table();
    builder.push_slot_always(VT_RECORD_BINARY_META, meta_off);
    builder.push_slot_always(VT_RECORD_BINARY_PAYLOAD, payload_off);
    let root = builder.end_table(start);
    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

fn build_record_crypt_bytes(
    meta: &Metadata,
    payload: &Payload,
    encrypt: &mut EncryptOptions,
) -> std::io::Result<Vec<u8>> {
    let attachment_bytes = build_attachment_bytes(payload)?;
    let encrypted_payload = tests::crypt::encrypt_bytes(&attachment_bytes, encrypt)?;
    let mut builder = FlatBufferBuilder::new();
    let meta_off = build_metadata_table(&mut builder, meta);
    let payload_off = builder.create_vector(encrypted_payload.as_slice());
    let start = builder.start_table();
    builder.push_slot_always(VT_RECORD_CRYPT_META, meta_off);
    builder.push_slot_always(VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED, payload_off);
    let root = builder.end_table(start);
    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

fn build_record_borrowed_bytes(block: &BlockBorrowed) -> std::io::Result<Vec<u8>> {
    let mut builder = FlatBufferBuilder::new();
    let field_u128_off = builder.create_vector(block.field_u128.to_le_bytes().as_slice());
    let field_i128_off = builder.create_vector(block.field_i128.to_le_bytes().as_slice());
    let blob_a_off = builder.create_vector(block.blob_a.as_slice());
    let blob_b_off = builder.create_vector(block.blob_b.as_slice());

    let borrowed_start = builder.start_table();
    builder.push_slot::<u8>(VT_BORROWED_FIELD_U8, block.field_u8, 0);
    builder.push_slot::<u16>(VT_BORROWED_FIELD_U16, block.field_u16, 0);
    builder.push_slot::<u32>(VT_BORROWED_FIELD_U32, block.field_u32, 0);
    builder.push_slot::<u64>(VT_BORROWED_FIELD_U64, block.field_u64, 0);
    builder.push_slot_always(VT_BORROWED_FIELD_U128, field_u128_off);
    builder.push_slot::<i8>(VT_BORROWED_FIELD_I8, block.field_i8, 0);
    builder.push_slot::<i16>(VT_BORROWED_FIELD_I16, block.field_i16, 0);
    builder.push_slot::<i32>(VT_BORROWED_FIELD_I32, block.field_i32, 0);
    builder.push_slot::<i64>(VT_BORROWED_FIELD_I64, block.field_i64, 0);
    builder.push_slot_always(VT_BORROWED_FIELD_I128, field_i128_off);
    builder.push_slot::<f32>(VT_BORROWED_FIELD_F32, block.field_f32, 0.0);
    builder.push_slot::<f64>(VT_BORROWED_FIELD_F64, block.field_f64, 0.0);
    builder.push_slot::<bool>(VT_BORROWED_FIELD_BOOL, block.field_bool, false);
    builder.push_slot_always(VT_BORROWED_BLOB_A, blob_a_off);
    builder.push_slot_always(VT_BORROWED_BLOB_B, blob_b_off);
    let borrowed_off = builder.end_table(borrowed_start);

    let root_start = builder.start_table();
    builder.push_slot_always(VT_RECORD_BORROWED_BLOCK, borrowed_off);
    let root = builder.end_table(root_start);
    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

pub(crate) fn record_bytes_from_packet(
    payload_kind: report::PayloadKind,
    packet: &WrappedPacket,
    encrypt: &mut EncryptOptions,
) -> std::io::Result<Vec<u8>> {
    match payload_kind {
        report::PayloadKind::Record => {
            let meta = metadata_from_packet(packet)?;
            let payload = payload_from_packet(packet)?;
            build_record_bytes(meta, payload)
        }
        report::PayloadKind::RecordBincode => {
            let meta = metadata_from_packet(packet)?;
            let payload = payload_from_packet(packet)?;
            build_record_binary_bytes(meta, payload)
        }
        report::PayloadKind::RecordCrypt => {
            let meta = metadata_from_packet(packet)?;
            let payload = payload_from_packet(packet)?;
            build_record_crypt_bytes(meta, payload, encrypt)
        }
        report::PayloadKind::Borrowed => {
            let block = borrowed_block_from_packet(packet)?;
            build_record_borrowed_bytes(block)
        }
    }
}
