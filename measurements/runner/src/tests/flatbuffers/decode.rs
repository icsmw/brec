use super::common::{
    FbTable, FbTableVector, VT_ATTACHMENT_CHUNK, VT_ATTACHMENT_DATA, VT_ATTACHMENT_FIELDS,
    VT_ATTACHMENT_NAME, VT_ATTACHMENT_UUID, VT_BORROWED_BLOB_A, VT_BORROWED_BLOB_B,
    VT_BORROWED_FIELD_BOOL, VT_BORROWED_FIELD_F32, VT_BORROWED_FIELD_F64, VT_BORROWED_FIELD_I8,
    VT_BORROWED_FIELD_I16, VT_BORROWED_FIELD_I32, VT_BORROWED_FIELD_I64, VT_BORROWED_FIELD_I128,
    VT_BORROWED_FIELD_U8, VT_BORROWED_FIELD_U16, VT_BORROWED_FIELD_U32, VT_BORROWED_FIELD_U64,
    VT_BORROWED_FIELD_U128, VT_FIELD_KEY, VT_FIELD_VALUE, VT_META_LEVEL, VT_META_TARGET,
    VT_META_TM, get_bool_default, get_f32_default, get_f64_default, get_i8_default,
    get_i16_default, get_i32_default, get_i64_default, get_required_bytes, get_required_str,
    get_required_table, get_required_u64, get_u8_default, get_u32_default, invalid_data,
};
use flatbuffers::ForwardsUOffset;
use std::collections::BTreeMap;

pub(crate) fn read_meta_level(root: &FbTable<'_>, slot: flatbuffers::VOffsetT) -> std::io::Result<u8> {
    let meta = get_required_table(root, slot, "missing meta table")?;
    get_u8_default(&meta, VT_META_LEVEL, 0)
}

pub(crate) fn touch_meta(root: &FbTable<'_>, slot: flatbuffers::VOffsetT) -> std::io::Result<()> {
    let meta = get_required_table(root, slot, "missing meta table")?;
    let _ = get_u8_default(&meta, VT_META_LEVEL, 0)?;
    let _ = get_u8_default(&meta, VT_META_TARGET, 0)?;
    let _ = get_required_u64(&meta, VT_META_TM, "missing meta.tm")?;
    Ok(())
}

pub(crate) fn touch_attachment(table: &FbTable<'_>) -> std::io::Result<()> {
    let _ = get_required_str(table, VT_ATTACHMENT_UUID, "missing attachment.uuid")?;
    let _ = get_required_str(table, VT_ATTACHMENT_NAME, "missing attachment.name")?;
    let _ = get_u32_default(table, VT_ATTACHMENT_CHUNK, 0)?;
    let _ = get_required_bytes(table, VT_ATTACHMENT_DATA, "missing attachment.data")?;

    let fields = unsafe { table.get::<ForwardsUOffset<FbTableVector<'_>>>(VT_ATTACHMENT_FIELDS, None) }
        .ok_or_else(|| invalid_data("missing attachment.fields"))?;
    for idx in 0..fields.len() {
        let field = fields.get(idx);
        let _ = get_required_str(&field, VT_FIELD_KEY, "missing field.key")?;
        let _ = get_required_str(&field, VT_FIELD_VALUE, "missing field.value")?;
    }
    Ok(())
}

pub(crate) fn attachment_name<'a>(table: &'a FbTable<'a>) -> std::io::Result<&'a str> {
    get_required_str(table, VT_ATTACHMENT_NAME, "missing attachment.name")
}

fn bytes_to_u128(bytes: &[u8], what: &'static str) -> std::io::Result<u128> {
    if bytes.len() != 16 {
        return Err(invalid_data(what));
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(bytes);
    Ok(u128::from_le_bytes(buf))
}

fn bytes_to_i128(bytes: &[u8], what: &'static str) -> std::io::Result<i128> {
    if bytes.len() != 16 {
        return Err(invalid_data(what));
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(bytes);
    Ok(i128::from_le_bytes(buf))
}

pub(crate) fn touch_borrowed_block(table: &FbTable<'_>) -> std::io::Result<()> {
    let _ = get_u8_default(table, VT_BORROWED_FIELD_U8, 0)?;
    let _ = unsafe { table.get::<u16>(VT_BORROWED_FIELD_U16, Some(0)) }
        .ok_or_else(|| invalid_data("invalid u16 slot"))?;
    let _ = get_u32_default(table, VT_BORROWED_FIELD_U32, 0)?;
    let _ = get_required_u64(table, VT_BORROWED_FIELD_U64, "missing borrowed.field_u64")?;
    let _ = bytes_to_u128(
        get_required_bytes(table, VT_BORROWED_FIELD_U128, "missing borrowed.field_u128")?,
        "invalid borrowed.field_u128",
    )?;
    let _ = get_i8_default(table, VT_BORROWED_FIELD_I8, 0)?;
    let _ = get_i16_default(table, VT_BORROWED_FIELD_I16, 0)?;
    let _ = get_i32_default(table, VT_BORROWED_FIELD_I32, 0)?;
    let _ = get_i64_default(table, VT_BORROWED_FIELD_I64, 0)?;
    let _ = bytes_to_i128(
        get_required_bytes(table, VT_BORROWED_FIELD_I128, "missing borrowed.field_i128")?,
        "invalid borrowed.field_i128",
    )?;
    let _ = get_f32_default(table, VT_BORROWED_FIELD_F32, 0.0)?;
    let _ = get_f64_default(table, VT_BORROWED_FIELD_F64, 0.0)?;
    let _ = get_bool_default(table, VT_BORROWED_FIELD_BOOL, false)?;
    let _ = get_required_bytes(table, VT_BORROWED_BLOB_A, "missing borrowed.blob_a")?;
    let _ = get_required_bytes(table, VT_BORROWED_BLOB_B, "missing borrowed.blob_b")?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct OwnedMeta {
    pub(crate) level: u8,
    pub(crate) target: u8,
    pub(crate) tm: u64,
}

#[derive(Debug)]
pub(crate) struct OwnedAttachment {
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) chunk: u32,
    pub(crate) data: Vec<u8>,
    pub(crate) fields: BTreeMap<String, String>,
}

#[derive(Debug)]
pub(crate) struct OwnedBorrowedBlock {
    pub(crate) field_u8: u8,
    pub(crate) field_u16: u16,
    pub(crate) field_u32: u32,
    pub(crate) field_u64: u64,
    pub(crate) field_u128: u128,
    pub(crate) field_i8: i8,
    pub(crate) field_i16: i16,
    pub(crate) field_i32: i32,
    pub(crate) field_i64: i64,
    pub(crate) field_i128: i128,
    pub(crate) field_f32: f32,
    pub(crate) field_f64: f64,
    pub(crate) field_bool: bool,
    pub(crate) blob_a: Vec<u8>,
    pub(crate) blob_b: Vec<u8>,
}

pub(crate) fn decode_meta_owned(table: &FbTable<'_>) -> std::io::Result<OwnedMeta> {
    Ok(OwnedMeta {
        level: get_u8_default(table, VT_META_LEVEL, 0)?,
        target: get_u8_default(table, VT_META_TARGET, 0)?,
        tm: get_required_u64(table, VT_META_TM, "missing meta.tm")?,
    })
}

pub(crate) fn decode_attachment_owned(table: &FbTable<'_>) -> std::io::Result<OwnedAttachment> {
    let uuid = get_required_str(table, VT_ATTACHMENT_UUID, "missing attachment.uuid")?.to_owned();
    let name = get_required_str(table, VT_ATTACHMENT_NAME, "missing attachment.name")?.to_owned();
    let chunk = get_u32_default(table, VT_ATTACHMENT_CHUNK, 0)?;
    let data = get_required_bytes(table, VT_ATTACHMENT_DATA, "missing attachment.data")?.to_vec();
    let fields = unsafe { table.get::<ForwardsUOffset<FbTableVector<'_>>>(VT_ATTACHMENT_FIELDS, None) }
        .ok_or_else(|| invalid_data("missing attachment.fields"))?;
    let mut entries = BTreeMap::new();
    for idx in 0..fields.len() {
        let field = fields.get(idx);
        entries.insert(
            get_required_str(&field, VT_FIELD_KEY, "missing field.key")?.to_owned(),
            get_required_str(&field, VT_FIELD_VALUE, "missing field.value")?.to_owned(),
        );
    }
    Ok(OwnedAttachment {
        uuid,
        name,
        chunk,
        data,
        fields: entries,
    })
}

pub(crate) fn decode_borrowed_owned(table: &FbTable<'_>) -> std::io::Result<OwnedBorrowedBlock> {
    Ok(OwnedBorrowedBlock {
        field_u8: get_u8_default(table, VT_BORROWED_FIELD_U8, 0)?,
        field_u16: unsafe { table.get::<u16>(VT_BORROWED_FIELD_U16, Some(0)) }
            .ok_or_else(|| invalid_data("invalid u16 slot"))?,
        field_u32: get_u32_default(table, VT_BORROWED_FIELD_U32, 0)?,
        field_u64: get_required_u64(table, VT_BORROWED_FIELD_U64, "missing borrowed.field_u64")?,
        field_u128: bytes_to_u128(
            get_required_bytes(table, VT_BORROWED_FIELD_U128, "missing borrowed.field_u128")?,
            "invalid borrowed.field_u128",
        )?,
        field_i8: get_i8_default(table, VT_BORROWED_FIELD_I8, 0)?,
        field_i16: get_i16_default(table, VT_BORROWED_FIELD_I16, 0)?,
        field_i32: get_i32_default(table, VT_BORROWED_FIELD_I32, 0)?,
        field_i64: get_i64_default(table, VT_BORROWED_FIELD_I64, 0)?,
        field_i128: bytes_to_i128(
            get_required_bytes(table, VT_BORROWED_FIELD_I128, "missing borrowed.field_i128")?,
            "invalid borrowed.field_i128",
        )?,
        field_f32: get_f32_default(table, VT_BORROWED_FIELD_F32, 0.0)?,
        field_f64: get_f64_default(table, VT_BORROWED_FIELD_F64, 0.0)?,
        field_bool: get_bool_default(table, VT_BORROWED_FIELD_BOOL, false)?,
        blob_a: get_required_bytes(table, VT_BORROWED_BLOB_A, "missing borrowed.blob_a")?.to_vec(),
        blob_b: get_required_bytes(table, VT_BORROWED_BLOB_B, "missing borrowed.blob_b")?.to_vec(),
    })
}
