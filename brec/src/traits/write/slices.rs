pub enum SliceSlot<'a> {
    Slice(&'a [u8]),
    Buf(Vec<u8>),
}
#[derive(Default)]
pub struct IoSlices<'a> {
    slots: Vec<SliceSlot<'a>>,
}

impl<'a> IoSlices<'a> {
    pub fn add_slice(&mut self, buf: &'a [u8]) {
        self.slots.push(SliceSlot::Slice(buf));
    }
    pub fn add_buffered(&mut self, buf: Vec<u8>) {
        self.slots.push(SliceSlot::Buf(buf));
    }
    pub fn get(&self) -> Vec<std::io::IoSlice> {
        self.slots
            .iter()
            .map(|slot| match slot {
                SliceSlot::Slice(buf) => std::io::IoSlice::new(buf),
                SliceSlot::Buf(buf) => std::io::IoSlice::new(buf),
            })
            .collect::<Vec<std::io::IoSlice>>()
    }
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
