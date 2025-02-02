pub use crate::*;

pub trait Signature {
    fn sig() -> &'static [u8; 4];
}

pub trait Crc {
    fn crc(&self) -> [u8; 4];
}

pub trait Size {
    fn size() -> u64;
}

pub trait Write: Crc + Size {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

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

pub enum ReadStatus<T> {
    DismatchSignature,
    Success(T),
    NotEnoughtData(u64),
    NotEnoughtDataToReadSig(u64),
}
pub trait TryRead {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}

pub trait TryReadBuffered {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}
