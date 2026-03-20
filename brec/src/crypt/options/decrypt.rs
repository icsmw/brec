use std::collections::VecDeque;
use std::path::Path;

use rsa::{Oaep, RsaPrivateKey, pkcs1::DecodeRsaPrivateKey, pkcs8::DecodePrivateKey};
use secrecy::{ExposeSecret, SecretBox};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::crypt::{
    consts,
    error::{CryptError, CryptResult},
    options::CryptPolicy,
};

/// Decryption settings for `BricCryptCodec`.
///
/// This object is designed to be reused and keeps already-parsed key material.
pub struct DecryptOptions {
    private_key: RsaPrivateKey,
    _private_key_source_pem: Option<SecretBox<String>>,
    expected_key_id: Option<Vec<u8>>,
    policy: CryptPolicy,
    session_cache: VecDeque<DecryptSessionCache>,
}

struct DecryptSessionCache {
    session_id: u64,
    key_id: Option<Vec<u8>>,
    wrapped_key_hash: [u8; 32],
    session_key: SecretBox<[u8; 32]>,
}

impl DecryptOptions {
    /// Creates options from an already parsed RSA private key.
    pub fn new(private_key: RsaPrivateKey) -> Self {
        Self {
            private_key,
            _private_key_source_pem: None,
            expected_key_id: None,
            policy: CryptPolicy::default(),
            session_cache: VecDeque::new(),
        }
    }

    /// Replaces crypto runtime policy for this options instance.
    pub fn with_policy(mut self, policy: CryptPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets optional key identifier expected in the crypto envelope.
    pub fn with_expected_key_id(mut self, key_id: impl Into<Vec<u8>>) -> Self {
        self.expected_key_id = match key_id.into() {
            key_id if key_id.is_empty() => None,
            key_id => Some(key_id),
        };
        self
    }

    /// Removes expected key identifier check.
    pub fn clear_expected_key_id(mut self) -> Self {
        self.expected_key_id = None;
        self
    }

    /// Returns configured key identifier, if set.
    pub fn expected_key_id(&self) -> Option<&[u8]> {
        self.expected_key_id.as_deref()
    }

    /// Returns parsed RSA private key.
    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private_key
    }

    /// Returns active runtime crypto policy.
    pub fn policy(&self) -> CryptPolicy {
        self.policy
    }

    pub(crate) fn cached_session_key(
        &self,
        session_id: u64,
        key_id: Option<&[u8]>,
        wrapped_key_hash: [u8; 32],
    ) -> Option<(usize, &[u8; 32])> {
        self.session_cache
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry.session_id == session_id
                    && entry.key_id.as_deref() == key_id
                    && entry.wrapped_key_hash == wrapped_key_hash
            })
            .map(|(idx, entry)| (idx, entry.session_key.expose_secret()))
    }

    pub(crate) fn remove_cache_session_by_idx(&mut self, idx: usize) {
        let _ = self.session_cache.remove(idx);
    }

    pub(crate) fn cache_session(
        &mut self,
        session_id: u64,
        key_id: Option<Vec<u8>>,
        wrapped_key_hash: [u8; 32],
        session_key: [u8; 32],
    ) {
        self.session_cache.retain(|entry| {
            !(entry.session_id == session_id
                && entry.key_id == key_id
                && entry.wrapped_key_hash == wrapped_key_hash)
        });
        self.session_cache.push_front(DecryptSessionCache {
            session_id,
            key_id,
            wrapped_key_hash,
            session_key: SecretBox::new(Box::new(session_key)),
        });
        while self.session_cache.len() > self.policy.decrypt_cache_limit {
            self.session_cache.pop_back();
        }
    }

    pub(crate) fn unwrap_session_key(
        &self,
        wrapped_key: &[u8],
    ) -> CryptResult<[u8; consts::ENVELOPE_SESSION_KEY_LEN]> {
        let mut session_key_raw = self
            .private_key()
            .decrypt(Oaep::new::<Sha256>(), wrapped_key)
            .map_err(|_| CryptError::UnwrapSessionKey)?;

        if session_key_raw.len() != consts::ENVELOPE_SESSION_KEY_LEN {
            session_key_raw.zeroize();
            return Err(CryptError::InvalidUnwrappedSessionKeyLength);
        }

        let mut session_key = [0u8; consts::ENVELOPE_SESSION_KEY_LEN];
        session_key.copy_from_slice(&session_key_raw);
        session_key_raw.zeroize();
        Ok(session_key)
    }

    #[cfg(test)]
    pub(crate) fn clear_session_cache(&mut self) {
        self.session_cache.clear();
    }

    /// Builds options from PEM that contains an RSA private key.
    pub fn from_private_key_pem(pem: &str) -> CryptResult<Self> {
        let private_key = parse_private_key_pem(pem)?;
        Ok(Self {
            private_key,
            _private_key_source_pem: Some(SecretBox::new(Box::new(pem.to_owned()))),
            expected_key_id: None,
            policy: CryptPolicy::default(),
            session_cache: VecDeque::new(),
        })
    }

    /// Builds options from PEM file that contains an RSA private key.
    pub fn from_private_key_pem_file(path: impl AsRef<Path>) -> CryptResult<Self> {
        let pem = read_text_file(path)?;
        Self::from_private_key_pem(&pem)
    }

    /// Alias for [`from_private_key_pem`].
    pub fn from_pem(pem: &str) -> CryptResult<Self> {
        Self::from_private_key_pem(pem)
    }

    /// Alias for [`from_private_key_pem_file`].
    pub fn from_pem_file(path: impl AsRef<Path>) -> CryptResult<Self> {
        Self::from_private_key_pem_file(path)
    }
}

fn parse_private_key_pem(pem: &str) -> CryptResult<RsaPrivateKey> {
    if let Ok(private_key) = RsaPrivateKey::from_pkcs8_pem(pem) {
        return Ok(private_key);
    }

    RsaPrivateKey::from_pkcs1_pem(pem).map_err(|_| CryptError::InvalidRsaPrivateKeyPem)
}

fn read_text_file(path: impl AsRef<Path>) -> CryptResult<String> {
    std::fs::read_to_string(path).map_err(CryptError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::{pkcs8::EncodePrivateKey, rand_core::OsRng, traits::PublicKeyParts};
    use std::io::Write;
    use tempfile::NamedTempFile;

    const TEST_EXPECTED_KEY_ID: &[u8] = b"kid-2";

    fn make_private_key_pem() -> String {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        private_key
            .to_pkcs8_pem(Default::default())
            .expect("private key pem")
            .to_string()
    }

    #[test]
    fn decrypt_options_expected_key_id_mutators() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let options = DecryptOptions::new(private_key)
            .with_expected_key_id(TEST_EXPECTED_KEY_ID.to_vec())
            .clear_expected_key_id();
        assert!(options.expected_key_id().is_none());
    }

    #[test]
    fn decrypt_options_empty_expected_key_id_is_normalized_to_none() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let options = DecryptOptions::new(private_key).with_expected_key_id(Vec::new());
        assert!(options.expected_key_id().is_none());
    }

    #[test]
    fn decrypt_options_from_private_key_pem_and_file() {
        let pem = make_private_key_pem();

        let from_pem = DecryptOptions::from_private_key_pem(&pem).expect("from pem");
        assert!(from_pem.expected_key_id().is_none());

        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(pem.as_bytes()).expect("write pem");
        let from_file = DecryptOptions::from_private_key_pem_file(file.path()).expect("from file");
        assert!(from_file.expected_key_id().is_none());
    }

    #[test]
    fn decrypt_options_from_pem_aliases_work() {
        let pem = make_private_key_pem();
        let options = DecryptOptions::from_pem(&pem).expect("from pem alias");
        assert!(options.private_key().to_public_key().n().bits() >= 1024);
    }

    #[test]
    fn decrypt_options_policy_mutator() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let policy = CryptPolicy {
            session_reuse_limit: 200,
            decrypt_cache_limit: 64,
        };
        let options = DecryptOptions::new(private_key).with_policy(policy);
        assert_eq!(options.policy(), policy);
    }
}
