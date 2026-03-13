use crate::crypt::error::CryptError;
use serde::{Deserialize, Serialize};

/// Supported crypto configuration identifiers embedded into wrapper bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum CryptAlgorithm {
    ChaCha20Poly1305RsaOaepSha256 = 1,
}

impl TryFrom<u8> for CryptAlgorithm {
    type Error = CryptError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ChaCha20Poly1305RsaOaepSha256),
            _ => Err(CryptError::UnsupportedAlgorithmId(value)),
        }
    }
}

impl From<CryptAlgorithm> for u8 {
    fn from(value: CryptAlgorithm) -> Self {
        value as u8
    }
}
