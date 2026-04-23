use crate::csharp_feat::{CSharpObject, CSharpValue, from_csharp_object, to_csharp_object};
use crate::*;

impl<B: BlockDef + CSharpObject, P: PayloadDef<Inner>, Inner: PayloadInnerDef + CSharpObject>
    PacketDef<B, P, Inner>
{
    /// Reads packet bytes and converts to `CSharpValue` object.
    pub fn decode_csharp(
        bytes: &[u8],
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<CSharpValue, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let packet = <Self as ReadPacketFrom>::read(&mut cursor, ctx)?;
        Ok(to_csharp_object(&packet.blocks, packet.payload.as_ref())?)
    }

    /// Parses `CSharpValue` packet object and encodes into packet bytes.
    pub fn encode_csharp(
        value: CSharpValue,
        out: &mut Vec<u8>,
        ctx: &mut <Inner as PayloadSchema>::Context<'_>,
    ) -> Result<(), Error> {
        let (blocks, payload) = from_csharp_object::<B, Inner>(value)?;
        let mut packet = Self::new(blocks, payload);
        packet.write_all(out, ctx)?;
        Ok(())
    }
}
