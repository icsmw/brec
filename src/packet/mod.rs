pub use crate::*;

pub trait Packet<'a, T> {
    fn sig() -> &'static [u8; 4];
    fn read(data: &'a [u8]) -> Result<Option<T>, Error>;
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
