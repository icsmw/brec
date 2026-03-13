use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use rsa::rand_core::{OsRng, RngCore};
use zeroize::Zeroize;

use crate::crypt::{
    algorithm::CryptAlgorithm,
    consts,
    error::{CryptError, CryptResult},
    options::{DecryptOptions, EncryptOptions},
    record::CryptEnvelopeRecord,
};
use crate::{PayloadDecode, PayloadEncode};

/// Codec for encrypting/decrypting payload bytes into internal crypto envelope.
pub struct BricCryptCodec;

impl BricCryptCodec {
    /// Encrypts payload bytes and serializes envelope.
    pub fn encrypt(payload_body: &[u8], options: &mut EncryptOptions) -> CryptResult<Vec<u8>> {
        let (session_id, wrapped_key, mut session_key) = match options.current_session() {
            Some((session_id, session_key, wrapped_key)) => {
                (session_id, wrapped_key.to_vec(), *session_key)
            }
            _ => options.create_session()?,
        };

        let mut nonce = [0u8; consts::ENVELOPE_NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        let cipher =
            ChaCha20Poly1305::new_from_slice(&session_key).map_err(|_| CryptError::InitCipher)?;

        session_key.zeroize();

        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), payload_body)
            .map_err(|_| CryptError::EncryptPayloadBody)?;

        CryptEnvelopeRecord::new(
            session_id,
            wrapped_key,
            nonce,
            ciphertext,
            options.key_id().map(Vec::from),
        )
        .encode()
    }

    /// Decrypts envelope bytes and returns plaintext payload bytes.
    pub fn decrypt(
        encrypted_payload_body: &[u8],
        options: &mut DecryptOptions,
    ) -> CryptResult<Vec<u8>> {
        fn decrypt_with_session_key(
            envelope: &CryptEnvelopeRecord,
            session_key: &[u8; consts::ENVELOPE_SESSION_KEY_LEN],
        ) -> CryptResult<Vec<u8>> {
            let cipher = ChaCha20Poly1305::new_from_slice(session_key)
                .map_err(|_| CryptError::InitCipher)?;
            cipher
                .decrypt(
                    Nonce::from_slice(&envelope.nonce),
                    envelope.payload.as_ref(),
                )
                .map_err(|_| CryptError::DecryptPayloadBody)
        }

        let envelope = CryptEnvelopeRecord::decode(encrypted_payload_body)?;
        if envelope.version != consts::ENVELOPE_VERSION {
            return Err(CryptError::UnsupportedEnvelopeVersion {
                actual: envelope.version,
                expected: consts::ENVELOPE_VERSION,
            });
        }

        if envelope.algorithm != CryptAlgorithm::ChaCha20Poly1305RsaOaepSha256 {
            return Err(CryptError::UnsupportedAlgorithm(envelope.algorithm));
        }

        if let Some(expected_key_id) = options.expected_key_id() {
            let key_id = envelope.key_id.as_deref().ok_or(CryptError::MissingKeyId)?;
            if key_id != expected_key_id {
                return Err(CryptError::KeyIdMismatch);
            }
        }

        let wrapped_key_hash = envelope.wrapped_key_hash();
        if let Some((idx, session_key)) = options.cached_session_key(
            envelope.session_id,
            envelope.key_id.as_deref(),
            wrapped_key_hash,
        ) {
            match decrypt_with_session_key(&envelope, session_key) {
                Ok(decrypted) => {
                    return Ok(decrypted);
                }
                Err(CryptError::DecryptPayloadBody) => {
                    // Fall back to envelope unwrap in case cache state is stale or collided.
                }
                Err(err) => {
                    options.remove_cache_session_by_idx(idx);
                    return Err(err);
                }
            }
        }

        let mut session_key = options.unwrap_session_key(&envelope.wrapped_key)?;

        let decrypted = decrypt_with_session_key(&envelope, &session_key)
            .inspect_err(|_| session_key.zeroize())?;

        options.cache_session(
            envelope.session_id,
            envelope.key_id.clone(),
            wrapped_key_hash,
            session_key,
        );
        session_key.zeroize();
        Ok(decrypted)
    }

    /// Parses envelope bytes without decrypting payload.
    pub fn parse(encrypted_payload_body: &[u8]) -> CryptResult<CryptEnvelopeRecord> {
        CryptEnvelopeRecord::decode(encrypted_payload_body)
    }

    /// Serializes already built envelope record.
    pub fn format(record: &CryptEnvelopeRecord) -> CryptResult<Vec<u8>> {
        record.encode()
    }

    /// Encodes a payload with default payload context and encrypts the bytes.
    pub fn encrypt_payload<T>(payload: &T, options: &mut EncryptOptions) -> std::io::Result<Vec<u8>>
    where
        T: PayloadEncode + crate::PayloadSchema<Context<'static> = crate::DefaultPayloadContext>,
    {
        let payload_body = payload.encode(&mut crate::default_payload_context())?;
        Self::encrypt(&payload_body, options).map_err(std::io::Error::from)
    }

    /// Decrypts bytes and decodes a payload with default payload context.
    pub fn decrypt_payload<T>(
        encrypted_payload_body: &[u8],
        options: &mut DecryptOptions,
    ) -> std::io::Result<T>
    where
        T: PayloadDecode<T> + crate::PayloadSchema<Context<'static> = crate::DefaultPayloadContext>,
    {
        let payload_body =
            Self::decrypt(encrypted_payload_body, options).map_err(std::io::Error::from)?;
        T::decode(&payload_body, &mut crate::default_payload_context())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::RsaPrivateKey;

    const TEST_KEY_ID: &[u8] = b"k1";
    const TEST_PAYLOAD: &[u8] = b"payload secret";

    #[test]
    fn crypt_codec_roundtrip() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();

        let mut encrypt_options = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let mut decrypt_options =
            DecryptOptions::new(private_key).with_expected_key_id(TEST_KEY_ID.to_vec());

        let encrypted =
            BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt");
        let decrypted = BricCryptCodec::decrypt(&encrypted, &mut decrypt_options).expect("decrypt");

        assert_eq!(decrypted, TEST_PAYLOAD);
    }

    #[test]
    fn crypt_codec_wrong_private_key() {
        let mut rng = OsRng;
        let source_private_key = RsaPrivateKey::new(&mut rng, 1024).expect("source key");
        let source_public_key = source_private_key.to_public_key();

        let another_private_key = RsaPrivateKey::new(&mut rng, 1024).expect("another key");

        let mut encrypt_options = EncryptOptions::new(source_public_key);
        let mut decrypt_options = DecryptOptions::new(another_private_key);

        let encrypted =
            BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt");
        let decrypted = BricCryptCodec::decrypt(&encrypted, &mut decrypt_options);

        assert!(decrypted.is_err());
    }

    #[test]
    fn envelope_record_roundtrip() {
        let record = CryptEnvelopeRecord::new(
            7,
            vec![1, 2, 3],
            [9; consts::ENVELOPE_NONCE_LEN],
            vec![4, 5, 6, 7],
            Some(vec![8, 8]),
        );
        let encoded = record.encode().expect("encode");
        let decoded = CryptEnvelopeRecord::decode(&encoded).expect("decode");
        assert_eq!(decoded, record);
    }

    #[derive(Debug, PartialEq)]
    struct MacroCryptPayload {
        value: u32,
    }

    impl crate::PayloadSchema for MacroCryptPayload {
        type Context<'a> = crate::DefaultPayloadContext;
    }

    impl crate::PayloadHooks for MacroCryptPayload {}

    impl crate::PayloadEncode for MacroCryptPayload {
        fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Ok(self.value.to_le_bytes().to_vec())
        }
    }

    impl crate::PayloadEncodeReferred for MacroCryptPayload {
        fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Ok(None)
        }
    }

    impl crate::PayloadDecode<MacroCryptPayload> for MacroCryptPayload {
        fn decode(buf: &[u8], _ctx: &mut Self::Context<'_>) -> std::io::Result<MacroCryptPayload> {
            let bytes: [u8; 4] = buf.try_into().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid payload len")
            })?;
            Ok(MacroCryptPayload {
                value: u32::from_le_bytes(bytes),
            })
        }
    }

    impl crate::PayloadSize for MacroCryptPayload {
        fn size(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<u64> {
            Ok(4)
        }
    }

    impl crate::PayloadCrc for MacroCryptPayload {}

    #[test]
    fn encrypt_payload_roundtrip_vec_u8() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();

        let mut encrypt_options = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let mut decrypt_options =
            DecryptOptions::new(private_key).with_expected_key_id(TEST_KEY_ID.to_vec());

        let payload = TEST_PAYLOAD.to_vec();
        let encrypted =
            BricCryptCodec::encrypt_payload(&payload, &mut encrypt_options).expect("encrypt");
        let decrypted =
            BricCryptCodec::decrypt_payload::<Vec<u8>>(&encrypted, &mut decrypt_options)
                .expect("decrypt");

        assert_eq!(decrypted, payload);
    }

    #[test]
    fn decrypt_payload_fails_on_key_id_mismatch() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();

        let mut encrypt_options = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let mut decrypt_options =
            DecryptOptions::new(private_key).with_expected_key_id(b"other".to_vec());

        let payload = TEST_PAYLOAD.to_vec();
        let encrypted =
            BricCryptCodec::encrypt_payload(&payload, &mut encrypt_options).expect("encrypt");
        let result = BricCryptCodec::decrypt_payload::<Vec<u8>>(&encrypted, &mut decrypt_options);

        assert!(result.is_err());
    }

    #[test]
    fn manual_payload_crypt_roundtrip() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();

        let payload = MacroCryptPayload { value: 42 };
        let mut encode_opt = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let encoded = BricCryptCodec::encrypt_payload(&payload, &mut encode_opt).expect("encode");

        let mut decode_opt =
            DecryptOptions::new(private_key).with_expected_key_id(TEST_KEY_ID.to_vec());
        let decoded =
            BricCryptCodec::decrypt_payload::<MacroCryptPayload>(&encoded, &mut decode_opt)
                .expect("decode");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn encrypt_reuses_session_for_configured_limit() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let mut encrypt_options = EncryptOptions::new(public_key);
        let reuse_limit = encrypt_options.policy().session_reuse_limit;

        let first = BricCryptCodec::parse(
            &BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt first"),
        )
        .expect("parse first");
        let second = BricCryptCodec::parse(
            &BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt second"),
        )
        .expect("parse second");

        assert_eq!(first.session_id, second.session_id);
        assert_eq!(first.wrapped_key, second.wrapped_key);

        let mut last = second;
        for _ in 2..reuse_limit {
            last = BricCryptCodec::parse(
                &BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt"),
            )
            .expect("parse");
        }

        let rotated = BricCryptCodec::parse(
            &BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt rotated"),
        )
        .expect("parse rotated");

        assert_ne!(last.session_id, rotated.session_id);
        assert_ne!(last.wrapped_key, rotated.wrapped_key);
    }

    #[test]
    fn encrypted_messages_remain_decryptable_without_decrypt_cache() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let mut encrypt_options = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let mut decrypt_options =
            DecryptOptions::new(private_key).with_expected_key_id(TEST_KEY_ID.to_vec());

        let payloads = [
            b"payload-1".as_slice(),
            b"payload-2".as_slice(),
            b"payload-3".as_slice(),
        ];

        let encrypted: Vec<Vec<u8>> = payloads
            .iter()
            .map(|payload| BricCryptCodec::encrypt(payload, &mut encrypt_options).expect("encrypt"))
            .collect();

        let parsed_first = BricCryptCodec::parse(&encrypted[0]).expect("parse first");
        let parsed_last = BricCryptCodec::parse(&encrypted[2]).expect("parse last");
        assert_eq!(parsed_first.session_id, parsed_last.session_id);

        for (encrypted, expected) in encrypted.iter().zip(payloads.iter()) {
            decrypt_options.clear_session_cache();
            let decrypted =
                BricCryptCodec::decrypt(encrypted, &mut decrypt_options).expect("decrypt");
            assert_eq!(&decrypted, expected);
        }
    }

    #[test]
    fn decrypt_falls_back_when_cached_session_key_is_stale() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("private key");
        let public_key = private_key.to_public_key();
        let mut encrypt_options = EncryptOptions::new(public_key).with_key_id(TEST_KEY_ID.to_vec());
        let mut decrypt_options =
            DecryptOptions::new(private_key).with_expected_key_id(TEST_KEY_ID.to_vec());

        let encrypted =
            BricCryptCodec::encrypt(TEST_PAYLOAD, &mut encrypt_options).expect("encrypt");
        let envelope = BricCryptCodec::parse(&encrypted).expect("parse");

        decrypt_options.cache_session(
            envelope.session_id,
            envelope.key_id.clone(),
            envelope.wrapped_key_hash(),
            [7u8; consts::ENVELOPE_SESSION_KEY_LEN],
        );

        let decrypted = BricCryptCodec::decrypt(&encrypted, &mut decrypt_options).expect("decrypt");
        assert_eq!(decrypted, TEST_PAYLOAD);
    }
}
