pub mod slices;

pub use slices::*;

use crate::prelude::*;

pub(crate) fn prepare_payload<T>(payload: &T) -> std::io::Result<(PayloadHeader, EncodedPayload<'_>)>
where
    T: PayloadSignature + PayloadEncode + PayloadEncodeReferred,
{
    let body = payload.encoded()?;
    let len = body.len();
    if len > u32::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Size of payload cannot be bigger {} bytes", u32::MAX),
        ));
    }

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(body.as_slice());

    let header = PayloadHeader {
        sig: payload.sig(),
        crc: ByteBlock::Len4(hasher.finalize().to_le_bytes()),
        len: len as u32,
    };

    Ok((header, body))
}

/// Trait for writing an immutable reference to a writable stream.
pub trait WriteTo {
    /// Writes the encoded contents to the given writer.
    ///
    /// # Returns
    /// The number of bytes written.
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;

    /// Writes all encoded content to the stream, ensuring complete output.
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

/// Trait for writing a mutable reference to a writable stream.
///
/// This is useful when the data to be written may require mutation during encoding.
pub trait WriteMutTo {
    /// Writes the encoded contents to the given writer.
    ///
    /// # Returns
    /// The number of bytes written.
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize>;

    /// Writes all encoded content to the stream, ensuring complete output.
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()>;
}

/// Trait for writing using vectored I/O with immutable data.
///
/// Vectored I/O can improve performance by writing multiple buffers at once.
pub trait WriteVectoredTo {
    /// Writes the encoded data using vectored I/O.
    ///
    /// # Returns
    /// Total number of bytes written from all slices.
    fn write_vectored<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    /// Returns a set of I/O slices representing the data to write.
    fn slices(&self) -> std::io::Result<IoSlices<'_>>;

    /// Writes all data using vectored I/O, ensuring that everything is written.
    fn write_vectored_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}

/// Trait for vectored I/O with mutable data.
///
/// This variant allows mutation when preparing data for writing.
pub trait WriteVectoredMutTo {
    /// Writes the encoded data using vectored I/O.
    fn write_vectored<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    /// Returns the I/O slices for the data to write.
    fn slices(&mut self) -> std::io::Result<IoSlices<'_>>;

    /// Ensures all data is written using vectored I/O.
    fn write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}

/// Trait for writing a payload with an automatically generated header.
///
/// This includes encoding the header and writing the encoded payload (either referred or standard).
pub trait WritePayloadWithHeaderTo
where
    Self: Sized
        + PayloadEncode
        + PayloadHooks
        + PayloadEncodeReferred
        + PayloadSignature
        + PayloadCrc
        + PayloadSize,
{
    /// Writes the header and payload to the output stream.
    ///
    /// # Returns
    /// The total number of bytes written.
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        let (header, body) = prepare_payload(self)?;
        let header = header.as_vec();
        buf.write_all(&header)?;
        buf.write(body.as_slice()).map(|s| s + header.len())
    }

    /// Writes the entire header and payload, ensuring completeness.
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        let (header, body) = prepare_payload(self)?;
        buf.write_all(&header.as_vec())?;
        buf.write_all(body.as_slice())
    }
}

/// Trait for writing a payload and header using vectored I/O.
///
/// Prepares both header and body into an `IoSlices` buffer for efficient writing.
pub trait WriteVectoredPayloadWithHeaderTo
where
    Self:
        Sized + PayloadEncode + PayloadEncodeReferred + PayloadSignature + PayloadCrc + PayloadSize,
{
    /// Writes the header and payload using vectored I/O.
    fn write_vectored<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    /// Prepares the header and payload slices for vectored I/O.
    fn slices(&mut self) -> std::io::Result<IoSlices<'_>> {
        let mut slices = IoSlices::default();
        let (header, body) = prepare_payload(self)?;
        let header = header.as_vec();
        slices.add_buffered(header.to_vec());
        match body {
            EncodedPayload::Borrowed(bytes) => slices.add_slice(bytes),
            EncodedPayload::Owned(bytes) => slices.add_buffered(bytes),
        }
        Ok(slices)
    }

    /// Writes all header and payload data using vectored I/O.
    fn write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}
