use thiserror::Error;

pub type CryptResult<T> = Result<T, CryptError>;

/// Unified error type for `brec::crypt` module.
#[derive(Debug, Error)]
pub enum CryptError {
    #[error("Invalid crypt input: {0}")]
    InvalidInput(String),
    #[error("Invalid crypt data: {0}")]
    InvalidData(String),
    #[error("Invalid RSA public key PEM")]
    InvalidRsaPublicKeyPem,
    #[error("Invalid PEM block")]
    InvalidPemBlock,
    #[error("Expected CERTIFICATE PEM block, got {actual}")]
    ExpectedCertificateBlock { actual: String },
    #[error("Invalid X509 certificate")]
    InvalidX509Certificate,
    #[error("Certificate doesn't contain valid RSA public key")]
    CertificateNoRsaPublicKey,
    #[error("Failed to export public key as PEM")]
    ExportPublicKeyPem,
    #[error("Invalid RSA private key PEM")]
    InvalidRsaPrivateKeyPem,
    #[error("Crypto envelope has empty key_id")]
    EmptyKeyId,
    #[error("Missing key_id in crypto envelope")]
    MissingKeyId,
    #[error("key_id mismatch in crypto envelope")]
    KeyIdMismatch,
    #[error("Failed to encrypt payload body")]
    EncryptPayloadBody,
    #[error("Failed to decrypt payload body")]
    DecryptPayloadBody,
    #[error("Unsupported crypt algorithm identifier: {0}")]
    UnsupportedAlgorithmId(u8),
    #[error("Unsupported crypt algorithm in envelope: {0:?}")]
    UnsupportedAlgorithm(crate::crypt::CryptAlgorithm),
    #[error("Unsupported crypt envelope version: got {actual}, expected {expected}")]
    UnsupportedEnvelopeVersion { actual: u8, expected: u8 },
    #[error("Failed to encode crypto envelope")]
    EncodeEnvelope,
    #[error("Failed to decode crypto envelope")]
    DecodeEnvelope,
    #[error("Malformed crypto envelope: decoded {decoded} bytes from {total}")]
    MalformedEnvelope { decoded: usize, total: usize },
    #[error("Failed to initialize chacha20poly1305 cipher")]
    InitCipher,
    #[error("Failed to wrap session key")]
    WrapSessionKey,
    #[error("Failed to unwrap session key")]
    UnwrapSessionKey,
    #[error("Invalid unwrapped session key length")]
    InvalidUnwrappedSessionKeyLength,
    #[error("One of sections is too large for envelope format: {0} bytes")]
    OversizedSection(usize),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl CryptError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData(message.into())
    }
}

impl From<CryptError> for std::io::Error {
    fn from(value: CryptError) -> Self {
        match value {
            CryptError::InvalidInput(message) => {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
            }
            CryptError::InvalidData(message) => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, message)
            }
            CryptError::InvalidRsaPublicKeyPem
            | CryptError::InvalidPemBlock
            | CryptError::ExpectedCertificateBlock { .. }
            | CryptError::InvalidX509Certificate
            | CryptError::CertificateNoRsaPublicKey
            | CryptError::ExportPublicKeyPem
            | CryptError::InvalidRsaPrivateKeyPem => {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, value.to_string())
            }
            CryptError::EmptyKeyId
            | CryptError::MissingKeyId
            | CryptError::KeyIdMismatch
            | CryptError::EncryptPayloadBody
            | CryptError::DecryptPayloadBody
            | CryptError::UnsupportedAlgorithmId(_)
            | CryptError::UnsupportedAlgorithm(_)
            | CryptError::UnsupportedEnvelopeVersion { .. }
            | CryptError::EncodeEnvelope
            | CryptError::DecodeEnvelope
            | CryptError::MalformedEnvelope { .. }
            | CryptError::InitCipher
            | CryptError::WrapSessionKey
            | CryptError::UnwrapSessionKey
            | CryptError::InvalidUnwrappedSessionKeyLength
            | CryptError::OversizedSection { .. } => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, value.to_string())
            }
            CryptError::Io(err) => err,
        }
    }
}
