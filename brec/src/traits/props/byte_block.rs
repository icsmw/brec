use crate::*;

/// A fixed-size byte buffer supporting several predefined lengths.
///
/// `ByteBlock` is used for representing binary data chunks with known allowed sizes.
/// It provides utility methods for accessing the internal data as a slice,
/// validating capacity, and converting from `Vec<u8>` safely.
#[derive(PartialEq, Eq, Debug)]
pub enum ByteBlock {
    /// 4-byte buffer.
    Len4([u8; 4]),
    /// 8-byte buffer.
    Len8([u8; 8]),
    /// 16-byte buffer.
    Len16([u8; 16]),
    /// 32-byte buffer.
    Len32([u8; 32]),
    /// 64-byte buffer.
    Len64([u8; 64]),
    /// 128-byte buffer.
    Len128([u8; 128]),
}

impl ByteBlock {
    /// Returns the internal byte array as a slice.
    ///
    /// # Returns
    /// A reference to the byte array stored in the `ByteBlock`
    pub fn as_slice(&self) -> &[u8] {
        match self {
            ByteBlock::Len4(arr) => arr,
            ByteBlock::Len8(arr) => arr,
            ByteBlock::Len16(arr) => arr,
            ByteBlock::Len32(arr) => arr,
            ByteBlock::Len64(arr) => arr,
            ByteBlock::Len128(arr) => arr,
        }
    }

    /// Returns the number of bytes stored in this block.
    pub fn size(&self) -> usize {
        self.as_slice().len()
    }

    /// Validates that the given capacity is allowed for a `ByteBlock`.
    ///
    /// Allowed sizes: 4, 8, 16, 32, 64, 128.
    ///
    /// # Arguments
    /// * `cap` – Capacity in bytes to validate.
    ///
    /// # Returns
    /// `Ok(())` if the capacity is valid, or an `Error::InvalidCapacity` otherwise.
    pub fn is_valid_capacity(cap: u8) -> Result<(), Error> {
        if [4, 8, 16, 32, 64, 128].contains(&cap) {
            Ok(())
        } else {
            Err(Error::InvalidCapacity(
                cap as usize,
                "4, 8, 16, 32, 64, 128".to_string(),
            ))
        }
    }
}

impl TryFrom<Vec<u8>> for ByteBlock {
    type Error = Error;

    /// Attempts to convert a `Vec<u8>` into a `ByteBlock` of matching length.
    ///
    /// # Arguments
    /// * `value` – The vector to convert. Length must be one of the supported sizes.
    ///
    /// # Returns
    /// A corresponding `ByteBlock` variant on success, or an error on failure:
    /// - `Error::InvalidCapacity` if the size is not supported.
    /// - `Error::FailExtractByteBlock` if conversion fails unexpectedly.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.len() {
            4 => value.try_into().map(Self::Len4),
            8 => value.try_into().map(Self::Len8),
            16 => value.try_into().map(Self::Len16),
            32 => value.try_into().map(Self::Len32),
            64 => value.try_into().map(Self::Len64),
            128 => value.try_into().map(Self::Len128),
            invalid => {
                return Err(Error::InvalidCapacity(
                    invalid,
                    "4, 8, 16, 32, 64, 128".to_string(),
                ))
            }
        }
        .map_err(|_| Error::FailExtractByteBlock)
    }
}
