/// Runtime limits that control crypto session reuse and cache sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CryptPolicy {
    /// How many times one generated session key may be reused for encryption.
    pub session_reuse_limit: u32,
    /// Maximum number of decrypted sessions cached for fast envelope reuse.
    pub decrypt_cache_limit: usize,
}

impl Default for CryptPolicy {
    fn default() -> Self {
        Self {
            session_reuse_limit: 100,
            decrypt_cache_limit: 32,
        }
    }
}
