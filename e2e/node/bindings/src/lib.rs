use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi::{Env, Unknown};
use napi_derive::napi;
use protocol::{Block, Packet, Payload};
use serde::{Deserialize, Serialize};

fn to_napi_error(prefix: &'static str, err: impl std::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, format!("{prefix}: {err}"))
}

#[napi]
pub fn decode_block<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    Block::decode_napi(env, buf).map_err(|e| to_napi_error("Decode block", e))
}

#[napi]
pub fn encode_block(_env: Env, val: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    Block::encode_napi(val, &mut buf).map_err(|e| to_napi_error("Encode block", e))?;
    Ok(buf.into())
}

#[napi]
pub fn decode_payload<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    let mut ctx = ();
    Payload::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode payload", e))
}

#[napi]
pub fn encode_payload(env: Env, val: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Payload::encode_napi(&env, val, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode payload", e))?;
    Ok(buf.into())
}

#[derive(Deserialize, Serialize)]
struct JsPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

impl From<Packet> for JsPacket {
    fn from(packet: Packet) -> Self {
        JsPacket {
            blocks: packet.blocks,
            payload: packet.payload,
        }
    }
}

#[napi]
pub fn decode_packet<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    let mut ctx = ();
    Packet::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode packet", e))
}

#[napi]
pub fn encode_packet(env: Env, blocks: Unknown<'_>, payload: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    let blocks = env
        .from_js_value::<Vec<Block>, _>(blocks)
        .map_err(|e| to_napi_error("Deserialize blocks", e))?;
    let payload = env
        .from_js_value::<Option<Payload>, _>(payload)
        .map_err(|e| to_napi_error("Deserialize payload", e))?;
    let packet_js = env
        .to_js_value(&JsPacket { blocks, payload })
        .map_err(|e| to_napi_error("Serialize packet", e))?;
    Packet::encode_napi(&env, packet_js, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet", e))?;
    Ok(buf.into())
}

#[napi]
pub fn encode_packet_object(env: Env, packet: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Packet::encode_napi(&env, packet, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet object", e))?;
    Ok(buf.into())
}

#[napi]
pub fn encode_packet_from_json(env: Env, packet_json: String) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    let packet: JsPacket = serde_json::from_str(&packet_json)
        .map_err(|e| to_napi_error("Deserialize packet json", e))?;
    let packet_js = env
        .to_js_value(&packet)
        .map_err(|e| to_napi_error("Serialize packet", e))?;
    Packet::encode_napi(&env, packet_js, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet", e))?;
    Ok(buf.into())
}
