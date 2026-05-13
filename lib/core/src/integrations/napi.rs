use crate::*;
use brec_node_lib::{
    NapiObject, from_napi_object,
    napi::{Env, Unknown, bindgen_prelude::Buffer},
    to_napi_object,
};

impl<B: BlockDef + NapiObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + NapiObject>
    PacketDef<B, P, Inner>
{
    /// Reads packet bytes and converts to JS object.
    pub fn decode_napi<'env>(
        env: &'env Env,
        bytes: Buffer,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<Unknown<'env>, Error> {
        let mut cursor = std::io::Cursor::new(bytes.as_ref());
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        Ok(to_napi_object(
            env,
            &packet.blocks,
            packet.payload.as_ref(),
        )?)
    }

    /// Parses JS object packet and encodes into packet bytes.
    pub fn encode_napi(
        env: &Env,
        value: Unknown<'_>,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let (blocks, payload) = from_napi_object::<B, Inner>(env, value)?;
        let mut packet = Self::new(blocks, payload);
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
