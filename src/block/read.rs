pub use crate::*;

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

impl<T> ReadStatus<T> {
    pub fn map<K, F>(self, mapper: F) -> ReadStatus<K>
    where
        F: FnOnce(T) -> K,
    {
        match self {
            ReadStatus::Success(value) => ReadStatus::Success(mapper(value)),
            ReadStatus::DismatchSignature => ReadStatus::DismatchSignature,
            ReadStatus::NotEnoughtData(n) => ReadStatus::NotEnoughtData(n),
            ReadStatus::NotEnoughtDataToReadSig(n) => ReadStatus::NotEnoughtDataToReadSig(n),
        }
    }
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
