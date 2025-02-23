use crate::*;

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteMutTo
    for PacketDef<B, P, Inner>
{
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        let mut total = header.write(buf)?;
        if total < PacketHeader::SIZE as usize {
            return Ok(total);
        }
        for blk in self.blocks.iter() {
            let size = blk.size() as usize;
            let written = blk.write(buf)?;
            if written < size {
                return Ok(total + written);
            }
            total += written;
        }
        if let Some(payload) = self.payload.as_mut() {
            let written = payload.write(buf)?;
            if written < payload.size()? as usize + PayloadHeader::LEN {
                return Ok(total + written);
            } else {
                total += written
            }
        }
        Ok(total)
    }
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        header.write_all(buf)?;
        for blk in self.blocks.iter() {
            blk.write_all(buf)?;
        }
        if let Some(payload) = self.payload.as_mut() {
            payload.write_all(buf)?;
        }
        Ok(())
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> WriteVectoredMutTo
    for PacketDef<B, P, Inner>
{
    fn slices(&mut self) -> std::io::Result<IoSlices> {
        let header = PacketHeader::new(&self.blocks, self.payload.as_ref())?;
        let mut slices = IoSlices::default();
        let mut header_bytes: Vec<u8> = Vec::new();
        header.write_all(&mut header_bytes)?;
        slices.add_buffered(header_bytes);
        for blk in self.blocks.iter() {
            slices.append(blk.slices()?);
        }
        if let Some(payload) = self.payload.as_mut() {
            slices.append(payload.slices()?);
        }
        Ok(slices)
    }
}
