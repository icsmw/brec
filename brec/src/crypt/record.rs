use sha2::{Digest, Sha256};

use crate::crypt::{
    algorithm::CryptAlgorithm,
    consts,
    error::{CryptError, CryptResult},
};

/// In-memory representation of encrypted payload wrapper bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CryptEnvelopeRecord {
    /// Envelope format version.
    pub version: u8,
    /// Encryption algorithm used for this envelope.
    pub algorithm: CryptAlgorithm,
    /// Session identifier used to correlate wrapped keys.
    pub session_id: u64,
    /// Wrapped symmetric session key bytes.
    pub wrapped_key: Vec<u8>,
    /// AEAD nonce used for payload encryption.
    pub nonce: [u8; consts::ENVELOPE_NONCE_LEN],
    /// Encrypted payload bytes (ciphertext + tag).
    pub payload: Vec<u8>,
    /// Optional key identifier carried with the envelope.
    pub key_id: Option<Vec<u8>>,
}

impl CryptEnvelopeRecord {
    /// Creates a new envelope record with crate defaults for version and algorithm.
    pub fn new(
        session_id: u64,
        wrapped_key: Vec<u8>,
        nonce: [u8; consts::ENVELOPE_NONCE_LEN],
        payload: Vec<u8>,
        key_id: Option<Vec<u8>>,
    ) -> Self {
        let key_id = key_id.filter(|key_id| !key_id.is_empty());
        Self {
            version: consts::ENVELOPE_VERSION,
            algorithm: CryptAlgorithm::ChaCha20Poly1305RsaOaepSha256,
            session_id,
            wrapped_key,
            nonce,
            payload,
            key_id,
        }
    }

    /// Serializes envelope record to bytes with bincode.
    pub fn encode(&self) -> CryptResult<Vec<u8>> {
        self.validate()?;
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| CryptError::EncodeEnvelope)
    }

    /// Parses envelope record from bincode bytes.
    pub fn decode(buf: &[u8]) -> CryptResult<Self> {
        let (record, read): (Self, usize) =
            bincode::serde::decode_from_slice(buf, bincode::config::standard())
                .map_err(|_| CryptError::DecodeEnvelope)?;
        if read != buf.len() {
            return Err(CryptError::MalformedEnvelope {
                decoded: read,
                total: buf.len(),
            });
        }
        record.validate()?;
        Ok(record)
    }

    pub(crate) fn wrapped_key_hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&Sha256::digest(&self.wrapped_key));
        hash
    }

    fn validate(&self) -> CryptResult<()> {
        let u32_max = |len: usize| -> CryptResult<()> {
            u32::try_from(len).map_err(|_| CryptError::OversizedSection(len))?;
            Ok(())
        };
        let u16_max = |len: usize| -> CryptResult<()> {
            u16::try_from(len).map_err(|_| CryptError::OversizedSection(len))?;
            Ok(())
        };
        u16_max(self.wrapped_key.len())?;
        u16_max(self.nonce.len())?;
        u16_max(self.key_id.as_deref().map(|v| v.len()).unwrap_or(0))?;
        u32_max(self.payload.len())?;
        self.key_id
            .as_deref()
            .map(|v| {
                if v.is_empty() {
                    Err(CryptError::EmptyKeyId)
                } else {
                    Ok(())
                }
            })
            .unwrap_or(Ok(()))?;
        Ok(())
    }
}
