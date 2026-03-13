#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CryptPolicy {
    pub session_reuse_limit: u32,
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
