use brec::prelude::*;

const TEST_KEY_ID: &[u8] = b"measurements-kid-1";

const PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDW+tQ57rfcwn7txFWEx9K1Td12
/aSyG/C5BqBsxd3qRsLXiEENZAFJBpMJJjxIi1Jy1f6syu07uTUBEJ4VyLqHCpEm
CoS8gy3UKdolQcGgFFaVnhjQkUrmhBWWzRQF+S2LrnJUWT7768u9pn/L/+Dfg7+3
shTKwbT6xqmEMhgzHwIDAQAB
-----END PUBLIC KEY-----"#;

const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIICdwIBADANBgkqhkiG9w0BAQEFAASCAmEwggJdAgEAAoGBANb61Dnut9zCfu3E
VYTH0rVN3Xb9pLIb8LkGoGzF3epGwteIQQ1kAUkGkwkmPEiLUnLV/qzK7Tu5NQEQ
nhXIuocKkSYKhLyDLdQp2iVBwaAUVpWeGNCRSuaEFZbNFAX5LYuuclRZPvvry72m
f8v/4N+Dv7eyFMrBtPrGqYQyGDMfAgMBAAECgYEAolgq+lDGskSCe+WfOzV3bokN
rHUg8Yvd/qv9bAcEbY3gR8lSbt1Nhysa0Hb6YUItEiF+QFjCTC6x00sMRqEeEcgy
8vXbT3HTf6b+Y5h3cfGgqjWyUpxkIUgduyLdWg0+ZM+RBAUtMurA9teefVI5k4G+
oAciPAO1jEDOfVL4jmECQQDxyKeZsiuSJWCZiCI/1cplzWbu/KM4dwq2lAIdbrOR
b4teUGhOF1GGOHt2wuQ4J/LnT4iewvAjDiTnjnHhRjlxAkEA4563/M8KwxhNmq+n
jO0EYVIJmXBlkfCypyqXRkgJ4v0iGy+jC8jtixUxT2yEdiEfqIv2VSlAxlv6GALw
v3ltjwJAf3QJvYBpbXHcmTJk84eMWNvM/gAZPmOqNxKQhtlbOTg2nHIAeeHa7MkL
dFBKI7wTVJHdb+tM0P3cwF+bcmglUQJBANMTzn6L6Oj+UojNt0yCRvuQvgIiLq5l
TOakImA0UabDIqufQ02caFv/rRiAA14gXWFJWYejl8Paa1N09pg5HJUCQGqAwNV8
0zZKU37X18Yy29FQTakpS8+3CdCwFIl8RQrbcvZ+6Ka10TxJu3k48A7pPwTiEe+6
8yDI4ZbntKmao2g=
-----END PRIVATE KEY-----"#;

pub fn encrypt_options(session_reuse_limit: u32, decrypt_cache_limit: usize) -> EncryptOptions {
    EncryptOptions::from_public_key_pem(PUBLIC_KEY_PEM)
        .expect("valid test public key")
        .with_policy(CryptPolicy {
            session_reuse_limit,
            decrypt_cache_limit,
        })
        .with_key_id(TEST_KEY_ID.to_vec())
}

pub fn decrypt_options(session_reuse_limit: u32, decrypt_cache_limit: usize) -> DecryptOptions {
    DecryptOptions::from_private_key_pem(PRIVATE_KEY_PEM)
        .expect("valid test private key")
        .with_policy(CryptPolicy {
            session_reuse_limit,
            decrypt_cache_limit,
        })
        .with_expected_key_id(TEST_KEY_ID.to_vec())
}

pub fn encrypt_bytes(payload: &[u8], options: &mut EncryptOptions) -> std::io::Result<Vec<u8>> {
    BricCryptCodec::encrypt(payload, options).map_err(std::io::Error::from)
}

pub fn decrypt_bytes(payload: &[u8], options: &mut DecryptOptions) -> std::io::Result<Vec<u8>> {
    BricCryptCodec::decrypt(payload, options).map_err(std::io::Error::from)
}
