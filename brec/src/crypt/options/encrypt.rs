use rsa::{
    Oaep, RsaPublicKey,
    pkcs1::DecodeRsaPublicKey,
    pkcs8::{DecodePublicKey, EncodePublicKey},
    rand_core::{OsRng, RngCore},
};
use secrecy::{ExposeSecret, SecretBox};
use sha2::Sha256;
use std::path::Path;
use x509_parser::prelude::parse_x509_certificate;
use zeroize::Zeroize;

use crate::crypt::{
    consts,
    error::{CryptError, CryptResult},
    options::CryptPolicy,
};

const PEM_CERTIFICATE_TAG: &str = "CERTIFICATE";

/// Encryption settings for `BricCryptCodec`.
///
/// This object is designed to be reused and keeps already-parsed key material.
pub struct EncryptOptions {
    public_key: RsaPublicKey,
    key_id: Option<Vec<u8>>,
    policy: CryptPolicy,
    session_cache: Option<EncryptSessionCache>,
}

pub(crate) struct EncryptSessionCache {
    session_id: u64,
    remaining_uses: u32,
    session_key: SecretBox<[u8; 32]>,
    wrapped_key: Vec<u8>,
}

impl EncryptOptions {
    /// Creates options from an already parsed RSA public key.
    pub fn new(public_key: RsaPublicKey) -> Self {
        Self {
            public_key,
            key_id: None,
            policy: CryptPolicy::default(),
            session_cache: None,
        }
    }

    /// Replaces crypto runtime policy for this options instance.
    pub fn with_policy(mut self, policy: CryptPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets optional key identifier embedded into the crypto envelope.
    pub fn with_key_id(mut self, key_id: impl Into<Vec<u8>>) -> Self {
        self.key_id = match key_id.into() {
            key_id if key_id.is_empty() => None,
            key_id => Some(key_id),
        };
        self
    }

    /// Removes key identifier from the envelope.
    pub fn clear_key_id(mut self) -> Self {
        self.key_id = None;
        self
    }

    /// Returns configured key identifier, if set.
    pub fn key_id(&self) -> Option<&[u8]> {
        self.key_id.as_deref()
    }

    /// Returns parsed RSA public key.
    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }

    pub fn policy(&self) -> CryptPolicy {
        self.policy
    }

    pub(crate) fn create_session(&mut self) -> CryptResult<(u64, Vec<u8>, [u8; 32])> {
        let mut session_id = OsRng.next_u64();
        if session_id == 0 {
            session_id = 1;
        }
        let mut session_key = [0u8; consts::ENVELOPE_SESSION_KEY_LEN];
        OsRng.fill_bytes(&mut session_key);
        let wrapped_key = self
            .public_key()
            .encrypt(&mut OsRng, Oaep::new::<Sha256>(), &session_key)
            .map_err(|_| {
                session_key.zeroize();
                CryptError::WrapSessionKey
            })?;
        self.cache_session(
            session_id,
            self.policy().session_reuse_limit.saturating_sub(1),
            session_key,
            wrapped_key.clone(),
        );
        Ok((session_id, wrapped_key, session_key))
    }

    pub(crate) fn current_session(&mut self) -> Option<(u64, &[u8; 32], &[u8])> {
        self.session_cache.as_mut().and_then(|session| {
            if session.remaining_uses == 0 {
                None
            } else {
                session.remaining_uses -= 1;
                Some((
                    session.session_id,
                    session.session_key.expose_secret(),
                    session.wrapped_key.as_slice(),
                ))
            }
        })
    }

    pub(crate) fn cache_session(
        &mut self,
        session_id: u64,
        remaining_uses: u32,
        session_key: [u8; 32],
        wrapped_key: Vec<u8>,
    ) {
        self.session_cache = Some(EncryptSessionCache {
            session_id,
            remaining_uses,
            session_key: SecretBox::new(Box::new(session_key)),
            wrapped_key,
        });
    }

    /// Builds options from PEM that contains an RSA public key.
    pub fn from_public_key_pem(pem: &str) -> CryptResult<Self> {
        let public_key = parse_public_key_pem(pem)?;
        Ok(Self::new(public_key))
    }

    /// Builds options from PEM file that contains an RSA public key.
    pub fn from_public_key_pem_file(path: impl AsRef<Path>) -> CryptResult<Self> {
        let pem = read_text_file(path)?;
        Self::from_public_key_pem(&pem)
    }

    /// Builds options from PEM that contains an X509 certificate with RSA public key.
    pub fn from_certificate_pem(certificate_pem: &str) -> CryptResult<Self> {
        let public_key = parse_certificate_public_key(certificate_pem)?;
        Ok(Self::new(public_key))
    }

    /// Builds options from PEM file that contains an X509 certificate with RSA public key.
    pub fn from_certificate_pem_file(path: impl AsRef<Path>) -> CryptResult<Self> {
        let pem = read_text_file(path)?;
        Self::from_certificate_pem(&pem)
    }

    /// Builds options from PEM that may contain either:
    /// - RSA public key
    /// - X509 certificate
    pub fn from_pem(pem: &str) -> CryptResult<Self> {
        match Self::from_public_key_pem(pem) {
            Ok(options) => Ok(options),
            Err(public_key_err) => Self::from_certificate_pem(pem).map_err(|_| public_key_err),
        }
    }

    /// Builds options from PEM file that may contain either:
    /// - RSA public key
    /// - X509 certificate
    pub fn from_pem_file(path: impl AsRef<Path>) -> CryptResult<Self> {
        let pem = read_text_file(path)?;
        Self::from_pem(&pem)
    }

    /// Exports configured public key as SPKI PEM.
    pub fn to_public_key_pem(&self) -> CryptResult<String> {
        self.public_key
            .to_public_key_pem(Default::default())
            .map_err(|_| CryptError::ExportPublicKeyPem)
    }
}

fn parse_public_key_pem(pem: &str) -> CryptResult<RsaPublicKey> {
    if let Ok(public_key) = RsaPublicKey::from_public_key_pem(pem) {
        return Ok(public_key);
    }

    RsaPublicKey::from_pkcs1_pem(pem).map_err(|_| CryptError::InvalidRsaPublicKeyPem)
}

fn parse_certificate_public_key(certificate_pem: &str) -> CryptResult<RsaPublicKey> {
    let pem = pem::parse(certificate_pem).map_err(|_| CryptError::InvalidPemBlock)?;

    if pem.tag() != PEM_CERTIFICATE_TAG {
        return Err(CryptError::ExpectedCertificateBlock {
            actual: pem.tag().to_owned(),
        });
    }

    let (_, cert) =
        parse_x509_certificate(pem.contents()).map_err(|_| CryptError::InvalidX509Certificate)?;

    RsaPublicKey::from_public_key_der(cert.public_key().raw)
        .map_err(|_| CryptError::CertificateNoRsaPublicKey)
}

fn read_text_file(path: impl AsRef<Path>) -> CryptResult<String> {
    std::fs::read_to_string(path).map_err(CryptError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::{RsaPrivateKey, pkcs8::EncodePublicKey, rand_core::OsRng};
    use std::io::Write;
    use tempfile::NamedTempFile;

    const TEST_KEY_ID: &[u8] = b"kid-1";

    fn make_public_key_pem() -> String {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        private_key
            .to_public_key()
            .to_public_key_pem(Default::default())
            .expect("public key pem")
    }

    #[test]
    fn encrypt_options_key_id_mutators() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let options = EncryptOptions::new(public_key)
            .with_key_id(TEST_KEY_ID.to_vec())
            .clear_key_id();
        assert!(options.key_id().is_none());
    }

    #[test]
    fn encrypt_options_empty_key_id_is_normalized_to_none() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let options = EncryptOptions::new(public_key).with_key_id(Vec::new());
        assert!(options.key_id().is_none());
    }

    #[test]
    fn encrypt_options_from_public_key_pem_and_file() {
        let pem = make_public_key_pem();

        let from_pem = EncryptOptions::from_public_key_pem(&pem).expect("from pem");
        assert!(from_pem.key_id().is_none());

        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(pem.as_bytes()).expect("write pem");
        let from_file = EncryptOptions::from_public_key_pem_file(file.path()).expect("from file");
        assert!(from_file.key_id().is_none());
    }

    #[test]
    fn encrypt_options_from_pem_falls_back_to_public_key() {
        let pem = make_public_key_pem();
        let options = EncryptOptions::from_pem(&pem).expect("from pem");
        let pem_exported = options.to_public_key_pem().expect("to pem");
        assert!(pem_exported.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn encrypt_options_from_pem_keeps_public_key_error_for_invalid_input() {
        let err = EncryptOptions::from_pem(
            "-----BEGIN PUBLIC KEY-----\nbroken\n-----END PUBLIC KEY-----",
        )
        .err()
        .expect("invalid public key");
        assert!(matches!(err, CryptError::InvalidRsaPublicKeyPem));
    }

    #[test]
    fn encrypt_options_policy_mutator() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let policy = CryptPolicy {
            session_reuse_limit: 200,
            decrypt_cache_limit: 64,
        };
        let options = EncryptOptions::new(public_key).with_policy(policy);
        assert_eq!(options.policy(), policy);
    }
}
