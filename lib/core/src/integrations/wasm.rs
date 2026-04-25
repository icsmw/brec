use crate::wasm_feat::{WasmObject, from_wasm_object, to_wasm_object, wasm_bindgen::JsValue};
use crate::*;

impl<B: BlockDef + WasmObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + WasmObject>
    PacketDef<B, P, Inner>
{
    /// Reads packet bytes and converts to JS object.
    pub fn decode_wasm(
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<JsValue, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        Ok(to_wasm_object(&packet.blocks, packet.payload.as_ref())?)
    }

    /// Parses JS object packet and encodes into packet bytes.
    pub fn encode_wasm(
        value: JsValue,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let (blocks, payload) = from_wasm_object::<B, Inner>(value)?;
        let mut packet = Self::new(blocks, payload);
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
