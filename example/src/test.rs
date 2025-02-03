use brec::*;

struct MyBlock {
    field: u8,
    log_level: u8,
}
#[repr(C)]
struct MyBlock1Referred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
}

impl<'a> From<MyBlock1Referred<'a>> for MyBlock {
    fn from(packet: MyBlock1Referred<'a>) -> Self {
        MyBlock {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYBLOCK: [u8; 4] = [183u8, 2u8, 127u8, 115u8];
impl Signature for MyBlock1Referred<'_> {
    fn sig() -> &'static [u8; 4] {
        &MYBLOCK
    }
}
impl brec::Crc for MyBlock {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for MyBlock {
    fn size() -> u64 {
        10u64
    }
}
impl brec::Read for MyBlock {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != MYBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut field = [0u8; 1];
        buf.read_exact(&mut field)?;
        let field = field[0];
        let mut log_level = [0u8; 1];
        buf.read_exact(&mut log_level)?;
        let log_level = log_level[0];
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let packet = MyBlock { field, log_level };
        if packet.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(packet)
    }
}
impl<'a> brec::ReadFromSlice<'a> for MyBlock1Referred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != MYBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            MyBlock::size() - 4
        } else {
            MyBlock::size()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughtData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &MYBLOCK
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let field = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(buf[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(buf[6usize..10usize].try_into()?);
        Ok(MyBlock1Referred {
            __sig,
            field,
            log_level,
            __crc,
        })
    }
}
impl brec::Write for MyBlock {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(buf.write(&MYBLOCK)?
            + buf.write(&[self.field])?
            + buf.write(&[self.log_level])?
            + buf.write(&self.crc())?)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYBLOCK)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}

impl brec::TryRead for MyBlock {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;

        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != MYBLOCK {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::DismatchSignature);
        }
        if len < MyBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(MyBlock::size() - len));
        }
        Ok(ReadStatus::Success(MyBlock::read(buf, true)?))
    }
}

impl brec::TryReadBuffered for MyBlock {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;

        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;

        if bytes.len() < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(
                (4 - bytes.len()) as u64,
            ));
        }

        if !bytes.starts_with(&MYBLOCK) {
            return Ok(ReadStatus::DismatchSignature);
        }

        if (bytes.len() as u64) < MyBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(
                MyBlock::size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = MyBlock::read(&mut reader, true);
        reader.consume(MyBlock::size() as usize - 4);
        Ok(ReadStatus::Success(blk?))
    }
}
pub(crate) enum Block {
    MyBlock(MyBlock),
}
