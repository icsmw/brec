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
}
