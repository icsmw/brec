use crate::*;

// TODO: needs a proptest
impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> ReadFrom
    for PacketDef<B, P, Inner>
{
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let header = PacketHeader::read(buf)?;
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            loop {
                // TODO: Error::SignatureDismatch should be covered in enum's context
                let blk = <B as ReadFrom>::read(buf)?;
                read += blk.size();
                pkg.blocks.push(blk);
                if read == header.blocks_len {
                    break;
                }
            }
        }
        if header.payload {
            let header = <PayloadHeader as ReadFrom>::read(buf)?;
            let payload = <P as ExtractPayloadFrom<Inner>>::read(buf, &header)?;
            pkg.payload = Some(payload);
        }
        Ok(pkg)
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFrom
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let start_pos = buf.stream_position()?;
        let available = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        let header = match <PacketHeader as TryReadFrom>::try_read(buf)? {
            ReadStatus::NotEnoughData(needed) => return Ok(ReadStatus::NotEnoughData(needed)),
            ReadStatus::Success(header) => header,
        };
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            loop {
                match <B as TryReadFrom>::try_read(buf) {
                    Ok(ReadStatus::Success(blk)) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    Ok(ReadStatus::NotEnoughData(needed)) => {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Ok(ReadStatus::NotEnoughData(needed));
                    }
                    Err(err) => {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Err(err);
                    }
                }
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFrom>::try_read(buf)? {
                ReadStatus::Success(header) => {
                    match <P as TryExtractPayloadFrom<Inner>>::try_read(buf, &header) {
                        Ok(ReadStatus::Success(payload)) => {
                            pkg.payload = Some(payload);
                        }
                        Ok(ReadStatus::NotEnoughData(needed)) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Ok(ReadStatus::NotEnoughData(needed));
                        }
                        Err(err) => {
                            buf.seek(std::io::SeekFrom::Start(start_pos))?;
                            return Err(err);
                        }
                    }
                }
                ReadStatus::NotEnoughData(needed) => {
                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    return Err(Error::NotEnoughData(needed as usize));
                }
            }
        }
        Ok(ReadStatus::Success(pkg))
    }
}

impl<B: BlockDef, P: PayloadDef<Inner>, Inner: PayloadInnerDef> TryReadFromBuffered
    for PacketDef<B, P, Inner>
{
    fn try_read<T: std::io::BufRead>(reader: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let bytes = reader.fill_buf()?;
        let available = bytes.len() as u64;
        if available < PacketHeader::ssize() {
            return Ok(ReadStatus::NotEnoughData(PacketHeader::ssize() - available));
        }
        let header = PacketHeader::read_from_slice(bytes, false)?;
        if header.size > available {
            return Ok(ReadStatus::NotEnoughData(header.size - available));
        }
        reader.consume(PacketHeader::ssize() as usize);
        let mut pkg = PacketDef::default();
        let mut read = 0;
        if header.blocks_len > 0 {
            loop {
                match <B as TryReadFromBuffered>::try_read(reader)? {
                    ReadStatus::Success(blk) => {
                        read += blk.size();
                        pkg.blocks.push(blk);
                        if read == header.blocks_len {
                            break;
                        }
                    }
                    ReadStatus::NotEnoughData(needed) => {
                        return Ok(ReadStatus::NotEnoughData(needed))
                    }
                }
            }
        }
        if header.payload {
            match <PayloadHeader as TryReadFromBuffered>::try_read(reader)? {
                ReadStatus::Success(header) => {
                    reader.consume(header.size());
                    match <P as TryExtractPayloadFromBuffered<Inner>>::try_read(reader, &header)? {
                        ReadStatus::Success(payload) => {
                            pkg.payload = Some(payload);
                        }
                        ReadStatus::NotEnoughData(needed) => {
                            return Ok(ReadStatus::NotEnoughData(needed))
                        }
                    }
                    reader.consume(header.payload_len());
                }
                ReadStatus::NotEnoughData(needed) => {
                    return Err(Error::NotEnoughData(needed as usize))
                }
            }
        }
        Ok(ReadStatus::Success(pkg))
    }
}
