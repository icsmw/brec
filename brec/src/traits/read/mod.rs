mod status;

use crate::*;
use payload::*;
pub use status::*;

pub trait ReadFrom {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ReadBlockFromSlice<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ReadBlockFrom {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ReadPayloadFrom {
    fn read<T: std::io::Read>(buf: &mut T, header: &PayloadHeader) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait TryReadPayloadFrom {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}

pub trait TryReadPayloadFromBuffered {
    fn try_read<T: std::io::Read>(
        buf: &mut T,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}

pub trait TryReadFrom {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}

pub trait TryReadFromBuffered {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized;
}
