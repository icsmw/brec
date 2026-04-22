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
    /// * `cap` - Capacity in bytes to validate.
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
    /// * `value` - The vector to convert. Length must be one of the supported sizes.
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
                ));
            }
        }
        .map_err(|_| Error::FailExtractByteBlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_slice_and_size_cover_all_variants() {
        let len4 = ByteBlock::Len4([1, 2, 3, 4]);
        assert_eq!(len4.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(len4.size(), 4);

        let len8 = ByteBlock::Len8([8; 8]);
        assert_eq!(len8.as_slice(), &[8; 8]);
        assert_eq!(len8.size(), 8);

        let len16 = ByteBlock::Len16([16; 16]);
        assert_eq!(len16.as_slice(), &[16; 16]);
        assert_eq!(len16.size(), 16);

        let len32 = ByteBlock::Len32([32; 32]);
        assert_eq!(len32.as_slice(), &[32; 32]);
        assert_eq!(len32.size(), 32);

        let len64 = ByteBlock::Len64([64; 64]);
        assert_eq!(len64.as_slice(), &[64; 64]);
        assert_eq!(len64.size(), 64);

        let len128 = ByteBlock::Len128([128; 128]);
        assert_eq!(len128.as_slice(), &[128; 128]);
        assert_eq!(len128.size(), 128);
    }

    #[test]
    fn try_from_vec_builds_expected_variant_for_supported_lengths() {
        let blk4 = ByteBlock::try_from(vec![4; 4]).expect("len4 should convert");
        assert!(matches!(blk4, ByteBlock::Len4(_)));
        assert_eq!(blk4.as_slice(), &[4; 4]);

        let blk8 = ByteBlock::try_from(vec![8; 8]).expect("len8 should convert");
        assert!(matches!(blk8, ByteBlock::Len8(_)));
        assert_eq!(blk8.as_slice(), &[8; 8]);

        let blk16 = ByteBlock::try_from(vec![16; 16]).expect("len16 should convert");
        assert!(matches!(blk16, ByteBlock::Len16(_)));
        assert_eq!(blk16.as_slice(), &[16; 16]);

        let blk32 = ByteBlock::try_from(vec![32; 32]).expect("len32 should convert");
        assert!(matches!(blk32, ByteBlock::Len32(_)));
        assert_eq!(blk32.as_slice(), &[32; 32]);

        let blk64 = ByteBlock::try_from(vec![64; 64]).expect("len64 should convert");
        assert!(matches!(blk64, ByteBlock::Len64(_)));
        assert_eq!(blk64.as_slice(), &[64; 64]);

        let blk128 = ByteBlock::try_from(vec![128; 128]).expect("len128 should convert");
        assert!(matches!(blk128, ByteBlock::Len128(_)));
        assert_eq!(blk128.as_slice(), &[128; 128]);
    }

    #[test]
    fn try_from_vec_rejects_unsupported_length() {
        let err = ByteBlock::try_from(vec![1; 3]).expect_err("len3 must be rejected");
        assert!(matches!(
            err,
            Error::InvalidCapacity(cap, expected)
                if cap == 3 && expected == "4, 8, 16, 32, 64, 128"
        ));
    }

    #[test]
    fn is_valid_capacity_accepts_supported_and_rejects_invalid() {
        for cap in [4_u8, 8, 16, 32, 64, 128] {
            assert!(ByteBlock::is_valid_capacity(cap).is_ok());
        }

        let err = ByteBlock::is_valid_capacity(7).expect_err("7 is unsupported");
        assert!(matches!(
            err,
            Error::InvalidCapacity(cap, expected)
                if cap == 7 && expected == "4, 8, 16, 32, 64, 128"
        ));
    }
}
