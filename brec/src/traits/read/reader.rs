/// A buffered reader that provides additional control over buffered data access.
///
/// Unlike the standard [`std::io::BufReader`], `BufferedReader` is specifically designed
/// to work with the `TryReadFromBuffered` trait, which requires checking if enough bytes
/// are available before attempting to read a `Block` or `Payload` signature.
/// If the buffer does not contain enough bytes to read the signature, reading
/// the `Block` or `Payload` without advancing the buffer position becomes impossible.
///
/// `BufferedReader` solves this issue by allowing safe "peeking" into the buffer before
/// consuming any data. This is achieved through the [`refill`] method, which preloads
/// the required amount of data to ensure the signature can be read safely.
///
/// ### Performance Considerations
///
/// - **Minimal Copying:** The design minimizes data copying, and in most cases, additional
///   copying is extremely rare. Copying occurs only in situations where fewer bytes are
///   available than required to read a signature.
/// - **Efficient Signature Reading:** Since `Block` signatures are typically 4 bytes long
///   and `Payload` signatures range from 4 (default) to 64 bytes, copying is only necessary
///   when the buffer contains fewer bytes than the required signature length.
/// - **No Performance Degradation:** In normal cases, `BufferedReader` operates
///   without introducing additional overhead compared to `std::io::BufReader`.
///
/// # Example
///
/// ```ignore
/// fn read_all_blocks(buffer: &[u8]) -> std::io::Result<(Vec<Block>, usize)> {
///     use brec::BufferedReader;
///     use std::io::Cursor;
///
///     let mut inner = Cursor::new(buffer);
///     let mut reader = BufferedReader::new(&mut inner);
///     let mut blocks = Vec::new();
///     loop {
///         if reader.buffer_len().unwrap() < 4 {
///             reader.refill().unwrap();
///         }
///         match <Block as TryReadFromBuffered>::try_read(&mut reader) {
///             Ok(ReadStatus::Success(blk)) => {
///                 blocks.push(blk);
///             }
///             Ok(ReadStatus::NotEnoughData(_needed)) => {
///                 reader.refill()?;
///                 break;
///             }
///             Err(err) => {
///                 return Err(std::io::Error::new(
///                     std::io::ErrorKind::InvalidData,
///                     err.to_string(),
///                 ));
///             }
///         }
///     }
///     Ok((blocks, reader.consumed()))
/// }
/// ```
pub struct BufferedReader<'a, R: std::io::BufRead> {
    inner: &'a mut R,
    buffer: Vec<u8>,
    filled: Vec<u8>,
    inner_len: usize,
    consumed: usize,
}

impl<'a, R: std::io::BufRead> BufferedReader<'a, R> {
    /// Creates a new `BufferedReader` wrapping an existing `BufRead` reader.
    ///
    /// The provided `inner` reader is used as the underlying buffer source.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn read_all_blocks(buffer: &[u8]) -> std::io::Result<(Vec<Block>, usize)> {
    ///     use brec::BufferedReader;
    ///     use std::io::Cursor;
    ///
    ///     let mut inner = Cursor::new(buffer);
    ///     let mut reader = BufferedReader::new(&mut inner);
    ///     let mut blocks = Vec::new();
    ///     loop {
    ///         if reader.buffer_len().unwrap() < 4 {
    ///             reader.refill().unwrap();
    ///         }
    ///         match <Block as TryReadFromBuffered>::try_read(&mut reader) {
    ///             Ok(ReadStatus::Success(blk)) => {
    ///                 blocks.push(blk);
    ///             }
    ///             Ok(ReadStatus::NotEnoughData(_needed)) => {
    ///                 reader.refill()?;
    ///                 break;
    ///             }
    ///             Err(err) => {
    ///                 return Err(std::io::Error::new(
    ///                     std::io::ErrorKind::InvalidData,
    ///                     err.to_string(),
    ///                 ));
    ///             }
    ///         }
    ///     }
    ///     Ok((blocks, reader.consumed()))
    /// }
    /// ```
    pub fn new(inner: &'a mut R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            filled: Vec::new(),
            inner_len: 0,
            consumed: 0,
        }
    }

    /// Returns the number of available bytes in the inner reader's buffer.
    ///
    /// This does not include bytes stored in `self.buffer`.
    pub fn buffer_len(&mut self) -> std::io::Result<usize> {
        Ok(self.inner.fill_buf()?.len())
    }

    /// Returns the number of available bytes in the inner reader's buffer and preload.
    ///
    /// This includes bytes stored in `self.buffer`.
    pub fn len(&mut self) -> std::io::Result<usize> {
        Ok(self.inner.fill_buf()?.len() + self.buffer.len())
    }

    /// Checks if buffer has something in inner and preload buffer
    pub fn is_empty(&mut self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Preloads data from the inner reader into `self.buffer`.
    ///
    /// This ensures that there are enough bytes available to read a signature
    /// before attempting to parse a `Block` or `Payload`.
    ///
    /// # Usage
    ///
    /// This method is typically used when there are insufficient bytes available
    /// in the buffer for reading a signature.
    ///
    /// ```ignore
    /// buffered_reader.refill()?;
    /// ```
    pub fn refill(&mut self) -> std::io::Result<()> {
        self.buffer = self.inner.fill_buf()?.to_vec();
        self.inner.consume(self.buffer.len());
        Ok(())
    }

    /// Returns amount of consumed bytes
    pub fn consumed(&self) -> usize {
        self.consumed
    }
}

impl<R: std::io::BufRead> std::io::Read for BufferedReader<'_, R> {
    /// Reads data from the buffer into `buf`, consuming buffered data first.
    ///
    /// - If `self.buffer` contains data, it is read first.
    /// - If additional data is needed, `self.inner.read()` is called.
    ///
    /// This ensures that any preloaded data is utilized before reading directly from
    /// the inner reader.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut total_read = 0;
        if !self.buffer.is_empty() {
            let to_copy = self.buffer.len().min(buf.len());
            buf[..to_copy].copy_from_slice(&self.buffer[..to_copy]);
            self.buffer.drain(..to_copy);
            total_read += to_copy;
        }
        if total_read < buf.len() {
            let from_inner = self.inner.read(&mut buf[total_read..])?;
            total_read += from_inner;
        }
        self.consumed += total_read;
        Ok(total_read)
    }
}

impl<R: std::io::BufRead> std::io::BufRead for BufferedReader<'_, R> {
    /// Returns a reference to the available buffered data.
    ///
    /// - If `self.buffer` is empty, it simply returns `self.inner.fill_buf()`.
    /// - If `self.buffer` contains data, it creates a merged buffer (`self.filled`)
    ///   combining `self.buffer` and the remaining bytes from `self.inner.fill_buf()`.
    ///
    /// This allows safe peeking into the available data before consuming it.
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.buffer.is_empty() {
            return self.inner.fill_buf();
        }
        let inner = self.inner.fill_buf()?;
        self.inner_len = inner.len();
        self.filled.clear();
        self.filled.reserve(inner.len() + self.buffer.len());
        self.filled.extend_from_slice(&self.buffer);
        self.filled.extend_from_slice(inner);
        Ok(&self.filled)
    }

    /// Consumes `amt` bytes from the buffer.
    ///
    /// - If `self.buffer` contains enough data, it is drained accordingly.
    /// - If `amt` exceeds `self.buffer.len()`, remaining bytes are consumed from `self.inner`.
    ///
    /// This ensures correct tracking of consumed data, preventing repeated reads.
    fn consume(&mut self, mut amt: usize) {
        if self.buffer.is_empty() {
            self.inner.consume(amt);
            self.consumed += amt;
            return;
        }
        self.filled.clear();

        let buf_len = self.buffer.len();
        if amt <= buf_len {
            self.buffer.drain(..amt);
            self.consumed += amt;
            return;
        }

        amt -= buf_len;
        self.buffer.clear();

        if amt <= self.inner_len {
            self.inner.consume(amt);
            self.inner_len -= amt;
            self.consumed += amt;
        } else {
            let leftover = amt - self.inner_len;
            self.inner_len = 0;
            self.inner.consume(leftover);
            self.consumed += leftover;
        }
    }
}
