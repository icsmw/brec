use crate::*;

pub trait ReadFromSlice<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait Read {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized;
}
