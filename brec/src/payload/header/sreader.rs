use crate::*;

pub enum NextChunk {
    NotEnoughData(u64),
    U8(u8),
    U32(u32),
    Bytes(Vec<u8>),
}

pub struct SafeHeaderReader<'a, T: std::io::Read + std::io::Seek> {
    buf: &'a mut T,
    spos: u64,
    read: u64,
    len: u64,
}

impl<'a, T: std::io::Read + std::io::Seek> SafeHeaderReader<'a, T> {
    pub fn new(buf: &'a mut T) -> Result<Self, Error> {
        let spos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - spos;
        buf.seek(std::io::SeekFrom::Start(spos))?;
        Ok(Self {
            spos,
            len,
            buf,
            read: 0,
        })
    }
    pub fn next_u8(&mut self) -> Result<NextChunk, Error> {
        if self.len < self.read + 1 {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(NextChunk::NotEnoughData(self.read + 1 - self.len));
        }
        let mut dest = [0u8; 1];
        self.buf.read_exact(&mut dest)?;
        self.read += 1;
        Ok(NextChunk::U8(dest[0]))
    }
    pub fn next_u32(&mut self) -> Result<NextChunk, Error> {
        if self.len < self.read + 4u64 {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(NextChunk::NotEnoughData(self.read + 4u64 - self.len));
        }
        let mut dest = [0u8; 4];
        self.buf.read_exact(&mut dest)?;
        self.read += 4u64;
        Ok(NextChunk::U32(u32::from_le_bytes(dest)))
    }
    pub fn next_bytes(&mut self, capacity: u64) -> Result<NextChunk, Error> {
        if self.len < self.read + capacity {
            self.buf.seek(std::io::SeekFrom::Start(self.spos))?;
            return Ok(NextChunk::NotEnoughData(self.read + capacity - self.len));
        }
        let mut dest = vec![0u8; capacity as usize];
        self.buf.read_exact(&mut dest)?;
        self.read += capacity;
        Ok(NextChunk::Bytes(dest))
    }
}
