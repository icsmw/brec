use crate::*;

/// Represents the result of a safe header read attempt.
///
/// Used by `SafeHeaderReader` to abstract over possibly incomplete input.
///
/// Instead of panicking or returning an error, this enum allows signaling how much
/// more data is required to proceed.
pub enum NextChunk {
    /// Not enough bytes were available to perform the read.
    ///
    /// Contains the number of missing bytes.
    NotEnoughData(u64),

    /// A successfully read 8-bit value.
    U8(u8),

    /// A successfully read 32-bit value.
    U32(u32),

    /// A successfully read raw byte buffer of the requested size.
    Bytes(Vec<u8>),
}

/// A utility reader for safely parsing fixed-size structures (e.g. payload headers).
///
/// `SafeHeaderReader` ensures that all reads are bounded by the remaining bytes in the stream.
/// If not enough data is available to complete a read, it returns `NextChunk::NotEnoughData(...)`
/// and resets the stream to the original start position.
///
/// # Usage
/// This is especially useful in packet inspection and header parsing, where early reads
/// must not consume data unless a full structure is available.
///
/// # Fields
/// - `buf`: The underlying readable+seekable stream
/// - `spos`: Original stream position before reading
/// - `read`: Number of bytes read so far
/// - `len`: Number of bytes available from `spos` to the end
pub struct SafeHeaderReader<'a, T: std::io::Read + std::io::Seek> {
    buf: &'a mut T,
    spos: u64,
    read: u64,
    len: u64,
}

impl<'a, T: std::io::Read + std::io::Seek> SafeHeaderReader<'a, T> {
    /// Constructs a new `SafeHeaderReader`, capturing the initial stream position.
    ///
    /// This also calculates the number of available bytes from the current position to the end.
    ///
    /// # Returns
    /// An initialized reader wrapper ready for safe, incremental reads.
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

    /// Attempts to read a single `u8` from the stream.
    ///
    /// If insufficient data is available, resets the stream and returns
    /// `NextChunk::NotEnoughData(1)`.
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

    /// Attempts to read a single `u32` (little-endian) from the stream.
    ///
    /// If fewer than 4 bytes are available, resets the stream and returns
    /// `NextChunk::NotEnoughData(needed)`.
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

    /// Attempts to read an arbitrary number of bytes from the stream.
    ///
    /// If not enough bytes are available, resets the stream and returns
    /// `NextChunk::NotEnoughData(needed)`.
    ///
    /// # Arguments
    /// * `capacity` – Number of bytes to read.
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
