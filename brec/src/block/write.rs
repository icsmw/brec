pub use crate::*;

pub trait Write: Crc + Size {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

pub trait WriteOwned: Crc + Size {
    fn write<T: std::io::Write>(self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(self, buf: &mut T) -> std::io::Result<()>;
}
