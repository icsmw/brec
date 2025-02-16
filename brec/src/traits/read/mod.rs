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

pub trait ReadPayloadFrom<T: Sized + PayloadDecode<T> + Signature + PayloadCrc> {
    fn read<B: std::io::Read>(buf: &mut B, header: &PayloadHeader) -> Result<T, Error>
    where
        Self: Sized + PayloadDecode<Self> + Signature,
    {
        if header.sig != T::sig() {
            return Err(Error::SignatureDismatch);
        }
        let mut bytes = vec![0u8; header.payload_len()];
        buf.read_exact(&mut bytes)?;
        let value = T::decode(&bytes)?;
        if header.crc != value.crc()? {
            return Err(Error::CrcDismatch);
        }
        Ok(value)
    }
}

pub trait ExtractPayloadFrom<T: Sized> {
    fn read<B: std::io::Read>(buf: &mut B, header: &PayloadHeader) -> Result<T, Error>;
}

pub trait TryReadPayloadFrom<
    T: Sized + PayloadDecode<T> + Signature + PayloadCrc + ReadPayloadFrom<T>,
>
{
    fn try_read<B: std::io::Read + std::io::Seek>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error> {
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < header.payload_len() as u64 {
            return Ok(ReadStatus::NotEnoughData(header.payload_len() as u64 - len));
        }
        <T as ReadPayloadFrom<T>>::read(buf, header).map(ReadStatus::Success)
    }
}

pub trait TryExtractPayloadFrom<T: Sized> {
    fn try_read<B: std::io::Read + std::io::Seek>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error>;
}

pub trait TryReadPayloadFromBuffered<
    T: Sized + PayloadDecode<T> + Signature + PayloadCrc + ReadPayloadFrom<T>,
>
{
    fn try_read<B: std::io::Read>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error> {
        <T as ReadPayloadFrom<T>>::read(buf, header).map(ReadStatus::Success)
    }
}

pub trait TryExtractPayloadFromBuffered<T: Sized + ExtractPayloadFrom<T>> {
    fn try_read<B: std::io::Read>(
        buf: &mut B,
        header: &PayloadHeader,
    ) -> Result<ReadStatus<T>, Error>;
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
