/// Helpers for building vectored write payload/body slices.
pub mod slices;

pub use slices::*;

use crate::prelude::*;

pub(crate) fn prepare_payload<'a, T>(
    payload: &'a T,
    ctx: &mut T::Context<'_>,
) -> std::io::Result<(PayloadHeader, EncodedPayload<'a>)>
where
    T: PayloadSignature + PayloadEncoded,
{
    let body = payload.encoded(ctx)?;
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
pub trait WriteMutTo: PayloadSchema {
    /// Writes the encoded contents to the given writer.
    ///
    /// # Returns
    /// The number of bytes written.
    fn write<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize>;

    /// Writes all encoded content to the stream, ensuring complete output.
    fn write_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()>;
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
pub trait WriteVectoredMutTo: PayloadSchema {
    /// Writes the encoded data using vectored I/O.
    fn write_vectored<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices(ctx)?.get())
    }

    /// Returns the I/O slices for the data to write.
    fn slices(
        &mut self,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<IoSlices<'_>>;

    /// Ensures all data is written using vectored I/O.
    fn write_vectored_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()> {
        self.slices(ctx)?.write_vectored_all(buf)
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
    fn write<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize> {
        let (header, body) = prepare_payload(self, ctx)?;
        let header = header.as_vec();
        buf.write_all(&header)?;
        buf.write(body.as_slice()).map(|s| s + header.len())
    }

    /// Writes the entire header and payload, ensuring completeness.
    fn write_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()> {
        let (header, body) = prepare_payload(self, ctx)?;
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
    fn write_vectored<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices(ctx)?.get())
    }

    /// Prepares the header and payload slices for vectored I/O.
    fn slices(
        &mut self,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<IoSlices<'_>> {
        let mut slices = IoSlices::default();
        let (header, body) = prepare_payload(self, ctx)?;
        let header = header.as_vec();
        slices.add_buffered(header.to_vec());
        match body {
            EncodedPayload::Borrowed(bytes) => slices.add_slice(bytes),
            EncodedPayload::Owned(bytes) => slices.add_buffered(bytes),
        }
        Ok(slices)
    }

    /// Writes all header and payload data using vectored I/O.
    fn write_vectored_all<T: std::io::Write>(
        &mut self,
        buf: &mut T,
        ctx: &mut <Self as PayloadSchema>::Context<'_>,
    ) -> std::io::Result<()> {
        self.slices(ctx)?.write_vectored_all(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PayloadEncode, PayloadEncodeReferred, PayloadHooks, PayloadSchema};
    use std::io::Cursor;

    struct DemoVectored<'a> {
        head: &'a [u8],
        tail: Vec<u8>,
    }

    impl WriteVectoredTo for DemoVectored<'_> {
        fn slices(&self) -> std::io::Result<IoSlices<'_>> {
            let mut slices = IoSlices::default();
            slices.add_slice(self.head);
            slices.add_buffered(self.tail.clone());
            Ok(slices)
        }
    }

    struct DemoVectoredMut<'a> {
        head: &'a [u8],
        tail: Vec<u8>,
    }

    impl PayloadSchema for DemoVectoredMut<'_> {
        type Context<'a> = ();
    }

    impl WriteVectoredMutTo for DemoVectoredMut<'_> {
        fn slices(
            &mut self,
            _: &mut <Self as PayloadSchema>::Context<'_>,
        ) -> std::io::Result<IoSlices<'_>> {
            let mut slices = IoSlices::default();
            slices.add_slice(self.head);
            slices.add_buffered(self.tail.clone());
            Ok(slices)
        }
    }

    struct BorrowedPayload {
        sig: [u8; 4],
        bytes: Vec<u8>,
    }

    impl PayloadSchema for BorrowedPayload {
        type Context<'a> = ();
    }

    impl PayloadHooks for BorrowedPayload {}

    impl PayloadEncode for BorrowedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.bytes.clone())
        }
    }

    impl PayloadEncodeReferred for BorrowedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(Some(self.bytes.as_slice()))
        }
    }

    impl PayloadSignature for BorrowedPayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(self.sig)
        }
    }

    impl PayloadCrc for BorrowedPayload {}
    impl PayloadSize for BorrowedPayload {}
    impl WritePayloadWithHeaderTo for BorrowedPayload {}
    impl WriteVectoredPayloadWithHeaderTo for BorrowedPayload {}

    struct OwnedPayload {
        sig: [u8; 4],
        bytes: Vec<u8>,
    }

    impl PayloadSchema for OwnedPayload {
        type Context<'a> = ();
    }

    impl PayloadHooks for OwnedPayload {}

    impl PayloadEncode for OwnedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.bytes.clone())
        }
    }

    impl PayloadEncodeReferred for OwnedPayload {
        fn encode(&self, _: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(None)
        }
    }

    impl PayloadSignature for OwnedPayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(self.sig)
        }
    }

    impl PayloadCrc for OwnedPayload {}
    impl PayloadSize for OwnedPayload {}
    impl WritePayloadWithHeaderTo for OwnedPayload {}
    impl WriteVectoredPayloadWithHeaderTo for OwnedPayload {}

    #[test]
    fn prepare_payload_and_payload_write_methods_work() {
        let mut payload = BorrowedPayload {
            sig: *b"TEST",
            bytes: vec![1, 2, 3, 4],
        };
        let mut ctx = ();

        let (header, body) = prepare_payload(&payload, &mut ctx).expect("prepare_payload");
        assert_eq!(header.len, 4);
        assert_eq!(header.sig.as_slice(), b"TEST");
        assert_eq!(body.as_slice(), &[1, 2, 3, 4]);

        let mut out = Cursor::new(Vec::new());
        let n = WritePayloadWithHeaderTo::write(&mut payload, &mut out, &mut ctx).expect("write");
        let bytes = out.into_inner();
        assert_eq!(n, bytes.len());
        assert_eq!(&bytes[..header.size()], header.as_vec().as_slice());
        assert_eq!(&bytes[header.size()..], &[1, 2, 3, 4]);

        let mut out = Cursor::new(Vec::new());
        WritePayloadWithHeaderTo::write_all(&mut payload, &mut out, &mut ctx).expect("write_all");
        assert_eq!(out.into_inner(), bytes);
    }

    #[test]
    fn vectored_payload_and_vectored_traits_write_all_data() {
        let mut borrowed = BorrowedPayload {
            sig: *b"BROW",
            bytes: vec![10, 11, 12],
        };
        let mut owned = OwnedPayload {
            sig: *b"OWND",
            bytes: vec![21, 22],
        };
        let mut ctx = ();

        let borrowed_slices = borrowed.slices(&mut ctx).expect("borrowed slices");
        assert_eq!(borrowed_slices.slots.len(), 2);
        assert!(matches!(borrowed_slices.slots[1], SliceSlot::Slice(_)));

        let owned_slices = owned.slices(&mut ctx).expect("owned slices");
        assert_eq!(owned_slices.slots.len(), 2);
        assert!(matches!(owned_slices.slots[1], SliceSlot::Buf(_)));

        let mut out = Cursor::new(Vec::new());
        let written = borrowed
            .write_vectored(&mut out, &mut ctx)
            .expect("borrowed write_vectored");
        assert_eq!(written, out.get_ref().len());

        let mut out_all = Cursor::new(Vec::new());
        owned
            .write_vectored_all(&mut out_all, &mut ctx)
            .expect("owned write_vectored_all");
        assert!(!out_all.into_inner().is_empty());

        let mut out_plain = Cursor::new(Vec::new());
        let item = DemoVectored {
            head: &[1, 2],
            tail: vec![3, 4],
        };
        let n = item.write_vectored(&mut out_plain).expect("write_vectored");
        assert_eq!(n, 4);
        assert_eq!(out_plain.into_inner(), vec![1, 2, 3, 4]);

        let mut out_mut = Cursor::new(Vec::new());
        let mut item_mut = DemoVectoredMut {
            head: &[5, 6],
            tail: vec![7, 8],
        };
        item_mut
            .write_vectored_all(&mut out_mut, &mut ())
            .expect("write_vectored_all");
        assert_eq!(out_mut.into_inner(), vec![5, 6, 7, 8]);
    }
}
