pub mod slices;

pub use slices::*;

use crate::prelude::*;

pub trait WriteTo {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

pub trait WriteMutTo {
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()>;
}

pub trait WriteVectoredTo {
    fn write_vectored<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    fn slices(&self) -> std::io::Result<IoSlices>;

    fn write_vectored_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}

pub trait WriteVectoredMutTo {
    fn write_vectored<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    fn slices(&mut self) -> std::io::Result<IoSlices>;

    fn write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}

pub trait WritePayloadWithHeaderTo
where
    Self:
        Sized + PayloadEncode + PayloadEncodeReferred + PayloadSignature + PayloadCrc + PayloadSize,
{
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        let header = PayloadHeader::new(self)?.as_vec();
        buf.write_all(&header)?;
        if let Some(bytes) = PayloadEncodeReferred::encode(self)? {
            buf.write(bytes)
        } else {
            buf.write(&PayloadEncode::encode(self)?)
        }
        .map(|s| s + header.len())
    }
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        let header = PayloadHeader::new(self)?.as_vec();
        buf.write_all(&header)?;
        if let Some(bytes) = PayloadEncodeReferred::encode(self)? {
            buf.write_all(bytes)
        } else {
            buf.write_all(&PayloadEncode::encode(self)?)
        }
    }
}

pub trait WriteVectoredPayloadWithHeaderTo
where
    Self:
        Sized + PayloadEncode + PayloadEncodeReferred + PayloadSignature + PayloadCrc + PayloadSize,
{
    fn write_vectored<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    fn slices(&mut self) -> std::io::Result<IoSlices> {
        let mut slices = IoSlices::default();
        let header = PayloadHeader::new(self)?.as_vec();
        slices.add_buffered(header.to_vec());
        if let Some(bytes) = PayloadEncodeReferred::encode(self)? {
            slices.add_slice(bytes);
        } else {
            slices.add_buffered(PayloadEncode::encode(self)?);
        }
        Ok(slices)
    }

    fn write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}
