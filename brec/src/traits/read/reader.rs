pub struct BufferedReader<'a, R: std::io::BufRead> {
    inner: &'a mut R,
    buffer: Vec<u8>,
    filled: Vec<u8>,
    inner_len: usize,
}

impl<'a, R: std::io::BufRead> BufferedReader<'a, R> {
    pub fn new(inner: &'a mut R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            filled: Vec::new(),
            inner_len: 0,
        }
    }
    pub fn buffer_len(&mut self) -> std::io::Result<usize> {
        Ok(self.inner.fill_buf()?.len())
    }
    pub fn refill(&mut self) -> std::io::Result<()> {
        self.buffer = self.inner.fill_buf()?.to_vec();
        self.inner.consume(self.buffer.len());
        Ok(())
    }
}

impl<R: std::io::BufRead> std::io::Read for BufferedReader<'_, R> {
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
        Ok(total_read)
    }
}

impl<R: std::io::BufRead> std::io::BufRead for BufferedReader<'_, R> {
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
    fn consume(&mut self, mut amt: usize) {
        if self.buffer.is_empty() {
            self.inner.consume(amt);
            return;
        }
        self.filled.clear();

        let buf_len = self.buffer.len();
        if amt <= buf_len {
            self.buffer.drain(..amt);
            return;
        }

        amt -= buf_len;
        self.buffer.clear();

        if amt <= self.inner_len {
            self.inner.consume(amt);
            self.inner_len -= amt;
        } else {
            let leftover = amt - self.inner_len;
            self.inner_len = 0;
            self.inner.consume(leftover);
        }
    }
}
