#[repr(C)]
struct BlockA {
    a: u32,
    b: u64,
    c: [u8; 100],
}
#[repr(C)]
struct BlockAReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    a: u32,
    b: u64,
    c: &'a [u8; 100usize],
    __crc: &'a [u8; 4usize],
}

impl<'a> From<BlockAReferred<'a>> for BlockA {
    fn from(block: BlockAReferred<'a>) -> Self {
        BlockA {
            a: block.a,
            b: block.b,
            c: *block.c,
        }
    }
}
const BLOCKA: [u8; 4] = [110u8, 88u8, 23u8, 102u8];
impl brec::SignatureU32 for BlockAReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &BLOCKA
    }
}
impl brec::CrcU32 for BlockA {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.a.to_le_bytes());
        hasher.update(&self.b.to_le_bytes());
        hasher.update(&self.c);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::CrcU32 for BlockAReferred<'_> {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.a.to_le_bytes());
        hasher.update(&self.b.to_le_bytes());
        hasher.update(self.c);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::StaticSize for BlockA {
    fn ssize() -> u64 {
        120u64
    }
}
impl brec::ReadBlockFrom for BlockA {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != BLOCKA {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut a = [0u8; 4usize];
        buf.read_exact(&mut a)?;
        let a = u32::from_le_bytes(a);
        let mut b = [0u8; 8usize];
        buf.read_exact(&mut b)?;
        let b = u64::from_le_bytes(b);
        let mut c = [0u8; 100usize];
        buf.read_exact(&mut c)?;
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = BlockA { a, b, c };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadBlockFromSlice<'a> for BlockAReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != BLOCKA {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            BlockA::ssize() - 4
        } else {
            BlockA::ssize()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &BLOCKA
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let a = u32::from_le_bytes(buf[4usize..8usize].try_into()?);
        let b = u64::from_le_bytes(buf[8usize..16usize].try_into()?);
        let c = <&[u8; 100usize]>::try_from(&buf[16usize..116usize])?;
        let __crc = <&[u8; 4usize]>::try_from(&buf[116usize..116usize + 4usize])?;
        let crc = __crc;
        let block = BlockAReferred {
            __sig,
            a,
            b,
            c,
            __crc,
        };
        if block.crc() != *crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl brec::TryReadFrom for BlockA {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(brec::ReadStatus::NotEnoughData(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != BLOCKA {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Err(brec::Error::SignatureDismatch);
        }
        if len < BlockA::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(BlockA::ssize() - len));
        }
        Ok(brec::ReadStatus::Success(BlockA::read(buf, true)?))
    }
}
impl brec::TryReadFromBuffered for BlockA {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(brec::ReadStatus::NotEnoughData((4 - bytes.len()) as u64));
        }
        if !bytes.starts_with(&BLOCKA) {
            return Err(brec::Error::SignatureDismatch);
        }
        if (bytes.len() as u64) < BlockA::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(
                BlockA::ssize() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = BlockA::read(&mut reader, true);
        reader.consume(BlockA::ssize() as usize - 4);
        Ok(brec::ReadStatus::Success(blk?))
    }
}
impl brec::WriteTo for BlockA {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKA);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.c.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKA);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.c.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}
impl brec::WriteVectoredTo for BlockA {
    fn slices(&self) -> std::io::Result<brec::IoSlices> {
        use brec::prelude::*;
        let mut slices = brec::IoSlices::default();
        slices.add_buffered(BLOCKA.to_vec());
        let mut buffer = [0u8; 12usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        slices.add_buffered(buffer.to_vec());
        slices.add_slice(&self.c);
        slices.add_buffered(self.crc().to_vec());
        Ok(slices)
    }
}
#[repr(C)]
struct BlockC {
    aa: i32,
    bb: i64,
    cc: [u8; 100],
}
#[repr(C)]
struct BlockCReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    aa: i32,
    bb: i64,
    cc: &'a [u8; 100usize],
    __crc: &'a [u8; 4usize],
}

impl<'a> From<BlockCReferred<'a>> for BlockC {
    fn from(block: BlockCReferred<'a>) -> Self {
        BlockC {
            aa: block.aa,
            bb: block.bb,
            cc: *block.cc,
        }
    }
}
const BLOCKC: [u8; 4] = [198u8, 121u8, 12u8, 80u8];
impl brec::SignatureU32 for BlockCReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &BLOCKC
    }
}
impl brec::CrcU32 for BlockC {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.aa.to_le_bytes());
        hasher.update(&self.bb.to_le_bytes());
        hasher.update(&self.cc);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::CrcU32 for BlockCReferred<'_> {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.aa.to_le_bytes());
        hasher.update(&self.bb.to_le_bytes());
        hasher.update(self.cc);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::StaticSize for BlockC {
    fn ssize() -> u64 {
        120u64
    }
}
impl brec::ReadBlockFrom for BlockC {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != BLOCKC {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut aa = [0u8; 4usize];
        buf.read_exact(&mut aa)?;
        let aa = i32::from_le_bytes(aa);
        let mut bb = [0u8; 8usize];
        buf.read_exact(&mut bb)?;
        let bb = i64::from_le_bytes(bb);
        let mut cc = [0u8; 100usize];
        buf.read_exact(&mut cc)?;
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = BlockC { aa, bb, cc };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadBlockFromSlice<'a> for BlockCReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != BLOCKC {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            BlockC::ssize() - 4
        } else {
            BlockC::ssize()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &BLOCKC
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let aa = i32::from_le_bytes(buf[4usize..8usize].try_into()?);
        let bb = i64::from_le_bytes(buf[8usize..16usize].try_into()?);
        let cc = <&[u8; 100usize]>::try_from(&buf[16usize..116usize])?;
        let __crc = <&[u8; 4usize]>::try_from(&buf[116usize..116usize + 4usize])?;
        let crc = __crc;
        let block = BlockCReferred {
            __sig,
            aa,
            bb,
            cc,
            __crc,
        };
        if block.crc() != *crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl brec::TryReadFrom for BlockC {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(brec::ReadStatus::NotEnoughData(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != BLOCKC {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Err(brec::Error::SignatureDismatch);
        }
        if len < BlockC::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(BlockC::ssize() - len));
        }
        Ok(brec::ReadStatus::Success(BlockC::read(buf, true)?))
    }
}
impl brec::TryReadFromBuffered for BlockC {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(brec::ReadStatus::NotEnoughData((4 - bytes.len()) as u64));
        }
        if !bytes.starts_with(&BLOCKC) {
            return Err(brec::Error::SignatureDismatch);
        }
        if (bytes.len() as u64) < BlockC::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(
                BlockC::ssize() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = BlockC::read(&mut reader, true);
        reader.consume(BlockC::ssize() as usize - 4);
        Ok(brec::ReadStatus::Success(blk?))
    }
}
impl brec::WriteTo for BlockC {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKC);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.aa.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.bb.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.cc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKC);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.aa.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.bb.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.cc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}
impl brec::WriteVectoredTo for BlockC {
    fn slices(&self) -> std::io::Result<brec::IoSlices> {
        use brec::prelude::*;
        let mut slices = brec::IoSlices::default();
        slices.add_buffered(BLOCKC.to_vec());
        let mut buffer = [0u8; 12usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&self.aa.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.bb.to_le_bytes());
        slices.add_buffered(buffer.to_vec());
        slices.add_slice(&self.cc);
        slices.add_buffered(self.crc().to_vec());
        Ok(slices)
    }
}
pub enum Block {
    BlockA(BlockA),
    BlockC(BlockC),
}
pub enum BlockReferred<'a> {
    BlockA(BlockAReferred<'a>),
    BlockC(BlockCReferred<'a>),
}
impl brec::BlockDef for Block {}
impl brec::Size for Block {
    fn size(&self) -> u64 {
        use brec::StaticSize;
        match self {
            Block::BlockA(..) => BlockA::ssize(),
            Block::BlockC(..) => BlockC::ssize(),
        }
    }
}
// impl<'a> brec::BlockReferredDef<'a, Block> for BlockReferred<'a> {}
impl brec::Size for BlockReferred<'_> {
    fn size(&self) -> u64 {
        use brec::StaticSize;
        match self {
            BlockReferred::BlockA(..) => BlockA::ssize(),
            BlockReferred::BlockC(..) => BlockC::ssize(),
        }
    }
}
impl brec::ReadFrom for Block {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::ReadBlockFrom>::read(buf, false) {
            Ok(blk) => return Ok(Block::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        match <BlockC as brec::ReadBlockFrom>::read(buf, false) {
            Ok(blk) => return Ok(Block::BlockC(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::ReadBlockFrom for Block {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::ReadBlockFrom>::read(buf, skip_sig) {
            Ok(blk) => return Ok(Block::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        match <BlockC as brec::ReadBlockFrom>::read(buf, skip_sig) {
            Ok(blk) => return Ok(Block::BlockC(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl<'a> brec::ReadBlockFromSlice<'a> for BlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match BlockAReferred::read_from_slice(buf, skip_sig) {
            Ok(blk) => return Ok(BlockReferred::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        match BlockCReferred::read_from_slice(buf, skip_sig) {
            Ok(blk) => return Ok(BlockReferred::BlockC(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::TryReadFrom for Block {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::TryReadFrom>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockA(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        match <BlockC as brec::TryReadFrom>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockC(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::TryReadFromBuffered for Block {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::TryReadFromBuffered>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockA(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        match <BlockC as brec::TryReadFromBuffered>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockC(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::WriteTo for Block {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        match self {
            Block::BlockA(blk) => blk.write(buf),
            Block::BlockC(blk) => blk.write(buf),
        }
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        match self {
            Block::BlockA(blk) => blk.write_all(buf),
            Block::BlockC(blk) => blk.write_all(buf),
        }
    }
}
impl brec::WriteVectoredTo for Block {
    fn slices(&self) -> std::io::Result<brec::IoSlices> {
        match self {
            Block::BlockA(blk) => blk.slices(),
            Block::BlockC(blk) => blk.slices(),
        }
    }
}
