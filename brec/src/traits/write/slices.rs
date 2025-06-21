/// A single slot in a vectored I/O buffer.
///
/// This enum represents a source of bytes to be written:
/// - either a borrowed slice (`&[u8]`)
/// - or an owned buffer (`Vec<u8>`)
pub enum SliceSlot<'a> {
    /// A borrowed byte slice.
    Slice(&'a [u8]),

    /// An owned byte buffer.
    Buf(Vec<u8>),
}

/// A collection of vectored I/O slices used for efficient streaming writes.
///
/// Internally maintains a list of `SliceSlot`s, which can be borrowed or owned.
/// Provides utilities to build and write the combined data.
#[derive(Default)]
pub struct IoSlices<'a> {
    /// Internal list of slices to be written.
    pub slots: Vec<SliceSlot<'a>>,
}

impl<'a> IoSlices<'a> {
    /// Adds a borrowed slice to the buffer.
    ///
    /// # Arguments
    /// * `buf` – A borrowed slice (`&[u8]`) to be written.
    pub fn add_slice(&mut self, buf: &'a [u8]) {
        self.slots.push(SliceSlot::Slice(buf));
    }

    /// Adds an owned buffer to the buffer list.
    ///
    /// # Arguments
    /// * `buf` – A `Vec<u8>` to be owned and written.
    pub fn add_buffered(&mut self, buf: Vec<u8>) {
        self.slots.push(SliceSlot::Buf(buf));
    }

    /// Converts all internal slots into a `Vec<IoSlice>` for writing.
    ///
    /// This method is non-destructive and safe to call multiple times.
    ///
    /// # Returns
    /// A vector of `std::io::IoSlice` values ready for `write_vectored`.
    pub fn get(&self) -> Vec<std::io::IoSlice<'_>> {
        self.slots
            .iter()
            .map(|slot| match slot {
                SliceSlot::Slice(buf) => std::io::IoSlice::new(buf),
                SliceSlot::Buf(buf) => std::io::IoSlice::new(buf),
            })
            .collect::<Vec<std::io::IoSlice>>()
    }

    /// Appends another `IoSlices` into this one, consuming the other buffer.
    ///
    /// # Arguments
    /// * `another` – Another `IoSlices` instance to merge into this one.
    pub fn append(&mut self, mut another: IoSlices<'a>) {
        self.slots.append(&mut another.slots);
    }

    /// Writes all buffers to the given stream using vectored I/O.
    ///
    /// Handles partial writes and retries until all slices are written.
    ///
    /// # Arguments
    /// * `buf` – A writable output stream.
    ///
    /// # Errors
    /// Returns an error if the write operation fails or stalls (`WriteZero`).
    pub fn write_vectored_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        let source = self.get();
        let mut written = 0;

        loop {
            let mut offset = 0;

            let in_work = source
                .iter()
                .filter_map(|slice| {
                    let start = offset;
                    let end = offset + slice.len();
                    offset = end;

                    if written >= end {
                        None
                    } else if written <= start {
                        Some(*slice)
                    } else {
                        let consumed = written - start;
                        Some(std::io::IoSlice::new(&slice[consumed..]))
                    }
                })
                .collect::<Vec<_>>();

            if in_work.is_empty() {
                break;
            }

            let just_written = buf.write_vectored(&in_work)?;
            written += just_written;

            if just_written == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "Failed to write data",
                ));
            }
        }

        Ok(())
    }
}
