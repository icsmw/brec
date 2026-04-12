mod err;

pub(crate) use err::*;

pub(crate) use protocol::{Block, Packet, Payload};
pub(crate) use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode_block(buf: &[u8]) -> Result<JsValue, ConvertorError> {
    Block::decode_wasm(buf).map_err(Into::into)
}

#[wasm_bindgen]
pub fn encode_block(val: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    Block::encode_wasm(val, &mut buf).map_err(ConvertorError::from)?;
    Ok(buf)
}

#[wasm_bindgen]
pub fn decode_payload(buf: &[u8]) -> Result<JsValue, ConvertorError> {
    let mut ctx = ();
    Payload::decode_wasm(buf, &mut ctx).map_err(Into::into)
}

#[wasm_bindgen]
pub fn encode_payload(val: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Payload::encode_wasm(val, &mut buf, &mut ctx).map_err(ConvertorError::from)?;
    Ok(buf)
}

#[wasm_bindgen]
pub fn decode_packet(buf: &[u8]) -> Result<JsValue, ConvertorError> {
    let mut ctx = ();
    Packet::decode_wasm(buf, &mut ctx).map_err(Into::into)
}

#[wasm_bindgen]
pub fn encode_packet(packet: JsValue) -> Result<Vec<u8>, ConvertorError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Packet::encode_wasm(packet, &mut buf, &mut ctx).map_err(ConvertorError::from)?;
    Ok(buf)
}
