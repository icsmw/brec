use flatbuffers::{ForwardsUOffset, Table, TableFinishedWIPOffset, VOffsetT, Vector, WIPOffset};

pub(crate) type FbTable<'a> = Table<'a>;
pub(crate) type FbTableOffset = WIPOffset<TableFinishedWIPOffset>;
pub(crate) type FbTableVector<'a> = Vector<'a, ForwardsUOffset<Table<'a>>>;

pub(crate) const VT_META_LEVEL: VOffsetT = 4;
pub(crate) const VT_META_TARGET: VOffsetT = 6;
pub(crate) const VT_META_TM: VOffsetT = 8;

pub(crate) const VT_FIELD_KEY: VOffsetT = 4;
pub(crate) const VT_FIELD_VALUE: VOffsetT = 6;

pub(crate) const VT_ATTACHMENT_UUID: VOffsetT = 4;
pub(crate) const VT_ATTACHMENT_NAME: VOffsetT = 6;
pub(crate) const VT_ATTACHMENT_CHUNK: VOffsetT = 8;
pub(crate) const VT_ATTACHMENT_DATA: VOffsetT = 10;
pub(crate) const VT_ATTACHMENT_FIELDS: VOffsetT = 12;

pub(crate) const VT_RECORD_META: VOffsetT = 4;
pub(crate) const VT_RECORD_MSG: VOffsetT = 6;

pub(crate) const VT_RECORD_BINARY_META: VOffsetT = 4;
pub(crate) const VT_RECORD_BINARY_PAYLOAD: VOffsetT = 6;

pub(crate) const VT_RECORD_CRYPT_META: VOffsetT = 4;
pub(crate) const VT_RECORD_CRYPT_PAYLOAD_ENCRYPTED: VOffsetT = 6;

pub(crate) const VT_BORROWED_FIELD_U8: VOffsetT = 4;
pub(crate) const VT_BORROWED_FIELD_U16: VOffsetT = 6;
pub(crate) const VT_BORROWED_FIELD_U32: VOffsetT = 8;
pub(crate) const VT_BORROWED_FIELD_U64: VOffsetT = 10;
pub(crate) const VT_BORROWED_FIELD_U128: VOffsetT = 12;
pub(crate) const VT_BORROWED_FIELD_I8: VOffsetT = 14;
pub(crate) const VT_BORROWED_FIELD_I16: VOffsetT = 16;
pub(crate) const VT_BORROWED_FIELD_I32: VOffsetT = 18;
pub(crate) const VT_BORROWED_FIELD_I64: VOffsetT = 20;
pub(crate) const VT_BORROWED_FIELD_I128: VOffsetT = 22;
pub(crate) const VT_BORROWED_FIELD_F32: VOffsetT = 24;
pub(crate) const VT_BORROWED_FIELD_F64: VOffsetT = 26;
pub(crate) const VT_BORROWED_FIELD_BOOL: VOffsetT = 28;
pub(crate) const VT_BORROWED_BLOB_A: VOffsetT = 30;
pub(crate) const VT_BORROWED_BLOB_B: VOffsetT = 32;

pub(crate) const VT_RECORD_BORROWED_BLOCK: VOffsetT = 4;

pub(crate) fn invalid_data(msg: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

pub(crate) fn fb_table<'a>(buf: &'a [u8]) -> FbTable<'a> {
    // Buffer is produced in this process by FlatBufferBuilder.
    unsafe { flatbuffers::root_unchecked::<Table<'a>>(buf) }
}

pub(crate) fn fb_table_safe<'a>(buf: &'a [u8]) -> std::io::Result<FbTable<'a>> {
    // "Safe"/owned mode in this benchmark means materializing owned Rust data.
    // It is not wire-level verification: dynamic table access here still uses unchecked root.
    Ok(fb_table(buf))
}

pub(crate) fn get_required_table<'a>(
    table: &FbTable<'a>,
    slot: VOffsetT,
    what: &'static str,
) -> std::io::Result<FbTable<'a>> {
    // Slot type is fixed by schema constants in this module.
    unsafe {
        table
            .get::<ForwardsUOffset<Table<'a>>>(slot, None)
            .ok_or_else(|| invalid_data(what))
    }
}

pub(crate) fn get_required_str<'a>(
    table: &FbTable<'a>,
    slot: VOffsetT,
    what: &'static str,
) -> std::io::Result<&'a str> {
    unsafe {
        table
            .get::<ForwardsUOffset<&str>>(slot, None)
            .ok_or_else(|| invalid_data(what))
    }
}

pub(crate) fn get_required_bytes<'a>(
    table: &FbTable<'a>,
    slot: VOffsetT,
    what: &'static str,
) -> std::io::Result<&'a [u8]> {
    unsafe {
        table
            .get::<ForwardsUOffset<&[u8]>>(slot, None)
            .ok_or_else(|| invalid_data(what))
    }
}

pub(crate) fn get_u8_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: u8,
) -> std::io::Result<u8> {
    unsafe {
        table
            .get::<u8>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid u8 slot"))
    }
}

pub(crate) fn get_u32_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: u32,
) -> std::io::Result<u32> {
    unsafe {
        table
            .get::<u32>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid u32 slot"))
    }
}

pub(crate) fn get_i8_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: i8,
) -> std::io::Result<i8> {
    unsafe {
        table
            .get::<i8>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid i8 slot"))
    }
}

pub(crate) fn get_i16_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: i16,
) -> std::io::Result<i16> {
    unsafe {
        table
            .get::<i16>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid i16 slot"))
    }
}

pub(crate) fn get_i32_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: i32,
) -> std::io::Result<i32> {
    unsafe {
        table
            .get::<i32>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid i32 slot"))
    }
}

pub(crate) fn get_i64_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: i64,
) -> std::io::Result<i64> {
    unsafe {
        table
            .get::<i64>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid i64 slot"))
    }
}

pub(crate) fn get_f32_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: f32,
) -> std::io::Result<f32> {
    unsafe {
        table
            .get::<f32>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid f32 slot"))
    }
}

pub(crate) fn get_f64_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: f64,
) -> std::io::Result<f64> {
    unsafe {
        table
            .get::<f64>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid f64 slot"))
    }
}

pub(crate) fn get_bool_default(
    table: &FbTable<'_>,
    slot: VOffsetT,
    default: bool,
) -> std::io::Result<bool> {
    unsafe {
        table
            .get::<bool>(slot, Some(default))
            .ok_or_else(|| invalid_data("invalid bool slot"))
    }
}

pub(crate) fn get_required_u64(
    table: &FbTable<'_>,
    slot: VOffsetT,
    what: &'static str,
) -> std::io::Result<u64> {
    unsafe { table.get::<u64>(slot, None).ok_or_else(|| invalid_data(what)) }
}
