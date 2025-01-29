pub use crate::*;

pub trait Block<'a, T> {
    fn sig() -> &'static [u8; 4];
}

pub trait Crc {
    fn crc(&self) -> [u8; 4];
}

pub trait Size {
    fn size(&self) -> usize;
}

pub trait Write: Crc + Size {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

pub trait ReadFromSlice<'a> {
    fn read_from_slice(buf: &'a [u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait Read {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized;
}
