use crate::java_feat::{
    JavaObject, from_java_object,
    jni::{JNIEnv, objects::JObject},
    to_java_object,
};
use crate::*;

impl<B: BlockDef + JavaObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + JavaObject>
    PacketDef<B, P, Inner>
{
    /// Reads packet bytes and converts to Java object.
    pub fn decode_java<'local>(
        env: &mut JNIEnv<'local>,
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<JObject<'local>, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        Ok(to_java_object(
            env,
            &packet.blocks,
            packet.payload.as_ref(),
        )?)
    }

    /// Parses Java object packet and encodes into packet bytes.
    pub fn encode_java<'local>(
        env: &mut JNIEnv<'local>,
        value: JObject<'local>,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let (blocks, payload) = from_java_object::<B, Inner>(env, value)?;
        let mut packet = Self::new(blocks, payload);
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
