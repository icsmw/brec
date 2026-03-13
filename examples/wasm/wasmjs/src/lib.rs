mod blocks;
mod err;
mod payloads;

pub(crate) use blocks::*;
pub(crate) use err::*;
pub(crate) use payloads::*;

pub(crate) use brec::prelude::*;
pub(crate) use serde::Serialize;
pub(crate) use serde_wasm_bindgen::{from_value, Serializer};
pub(crate) use wasm_bindgen::prelude::*;

brec::generate!();

/// Attempts to decode the provided byte slice into a `Block`.
///
/// # Arguments
/// * `buf` - A byte slice containing the serialized block data.
///
/// # Returns
/// On success, returns the decoded block as a `JsValue`, suitable for use in JavaScript.
/// On failure, returns a `ConvertorError`.
#[wasm_bindgen]
pub fn decode_block(mut buf: &[u8]) -> Result<JsValue, ConvertorError> {
    let serializer = Serializer::new()
        .serialize_missing_as_null(true)
        .serialize_maps_as_objects(false)
        .serialize_large_number_types_as_bigints(false);
    let block = <Block as ReadBlockFrom>::read(&mut buf, false)
        .map_err(|e| ConvertorError::ReadError(e.to_string()))?;
    block
        .serialize(&serializer)
        .map_err(|e| ConvertorError::SerializeError(e.to_string()))
}

/// Encodes the provided block into a byte vector, adding the block signature and CRC.
///
/// # Arguments
/// * `val` - A `JsValue` representing the block object passed from JavaScript.
///
/// # Returns
/// A `Vec<u8>` containing the serialized block, including the signature and CRC.
/// On error, returns a `ConvertorError`.
#[wasm_bindgen]
pub fn encode_block(val: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    from_value::<Block>(val)?
        .write_all(&mut buf)
        .map_err(|e| ConvertorError::WriteError(e.to_string()))?;
    Ok(buf)
}

/// Attempts to decode the provided byte slice into a `Payload`.
///
/// # Arguments
/// * `buf` - A byte slice containing the serialized payload data.
///
/// # Returns
/// On success, returns the decoded payload as a `JsValue`, suitable for use in JavaScript.
/// On failure, returns a `ConvertorError`.
#[wasm_bindgen]
pub fn decode_payload(buf: &[u8]) -> Result<JsValue, ConvertorError> {
    let serializer = Serializer::new()
        .serialize_missing_as_null(true)
        .serialize_maps_as_objects(false)
        .serialize_large_number_types_as_bigints(false);
    let mut cursor = std::io::Cursor::new(buf);
    let header = PayloadHeader::read(&mut cursor)
        .map_err(|e| ConvertorError::PayloadHeaderReading(e.to_string()))?;
    let payload = Payload::read(&mut cursor, &header)
        .map_err(|e| ConvertorError::ReadError(e.to_string()))?;
    payload
        .serialize(&serializer)
        .map_err(|e| ConvertorError::SerializeError(e.to_string()))
}

/// Encodes the provided payload into a byte vector, including the payload header,
/// which contains the signature, CRC, and length information.
///
/// # Arguments
/// * `val` - A `JsValue` representing the payload object passed from JavaScript.
///
/// # Returns
/// A `Vec<u8>` containing the serialized payload, including its header.
/// On error, returns a `ConvertorError`.
#[wasm_bindgen]
pub fn encode_payload(val: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    from_value::<Payload>(val)?
        .write_all(&mut buf)
        .map_err(|e| ConvertorError::WriteError(e.to_string()))?;
    Ok(buf)
}

/// JavaScript representation of a `Packet`.
///
/// Contains a list of blocks and an optional payload.
#[derive(serde::Deserialize, serde::Serialize)]
struct JsPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

/// Converts a Rust `Packet` into its JavaScript representation (`JsPacket`).
impl From<Packet> for JsPacket {
    fn from(packet: Packet) -> Self {
        JsPacket {
            blocks: packet.blocks,
            payload: packet.payload,
        }
    }
}

/// Parses a byte slice to reconstruct a `Packet`.
///
/// # Arguments
/// * `buf` - A byte slice containing the serialized packet data.
///
/// # Returns
/// On success, returns a `JsValue` representing the deserialized packet (as a `JsPacket`) for use in JavaScript.
/// If the data is incomplete, returns a `ConvertorError`.
/// If there's excess data, the packet is still returned, but the number of bytes read is not exposed.
#[wasm_bindgen]
pub fn decode_packet(buf: &[u8]) -> Result<JsValue, ConvertorError> {
    let serializer = Serializer::new()
        .serialize_missing_as_null(true)
        .serialize_maps_as_objects(false)
        .serialize_large_number_types_as_bigints(false);
    let mut cursor = std::io::Cursor::new(buf);
    let packet: JsPacket = Packet::read(&mut cursor)
        .map_err(|e| ConvertorError::ReadError(e.to_string()))?
        .into();
    packet
        .serialize(&serializer)
        .map_err(|e| ConvertorError::SerializeError(e.to_string()))
}

/// Creates a `Packet` from a list of blocks and an optional payload, then serializes it into bytes.
///
/// # Arguments
/// * `blocks` - A `JsValue` representing the list of blocks from JavaScript.
/// * `payload` - A `JsValue` representing the optional payload from JavaScript.
///
/// # Returns
/// A `Vec<u8>` containing the serialized packet data.
/// On error, returns a `ConvertorError`.
#[wasm_bindgen]
pub fn encode_packet(blocks: JsValue, payload: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    let blocks = from_value::<Vec<Block>>(blocks)?;
    let payload = from_value::<Option<Payload>>(payload)?;
    Packet::new(blocks, payload)
        .write_all(&mut buf)
        .map_err(|e| ConvertorError::WriteError(e.to_string()))?;
    Ok(buf)
}
