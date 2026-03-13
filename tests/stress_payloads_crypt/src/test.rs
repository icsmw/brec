use brec::prelude::*;
use proptest::prelude::*;
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
    rand_core::OsRng,
};
use std::sync::OnceLock;

use crate::*;

brec::generate!();

impl Arbitrary for Payload {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            PayloadA::arbitrary().prop_map(Payload::PayloadA),
            PayloadB::arbitrary().prop_map(Payload::PayloadB),
            PayloadC::arbitrary().prop_map(Payload::PayloadC),
            PayloadD::arbitrary().prop_map(Payload::PayloadD),
        ]
        .boxed()
    }
}

const KEY_ID: &[u8] = b"stress_payloads_crypt_key";

struct TestCertificates {
    encrypt: EncryptOptions,
    decrypt: DecryptOptions,
    wrong_decrypt: DecryptOptions,
}

fn certificates() -> TestCertificates {
    static CERTS: OnceLock<(String, String, String)> = OnceLock::new();
    let (public_key_pem, private_key_pem, wrong_private_key_pem) = CERTS.get_or_init(|| {
        let mut rng = OsRng;

        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("private key");
        let public_key_pem = private_key
            .to_public_key()
            .to_public_key_pem(Default::default())
            .expect("public key pem");
        let private_key_pem = private_key
            .to_pkcs8_pem(Default::default())
            .expect("private key pem")
            .to_string();
        let wrong_private_key = RsaPrivateKey::new(&mut rng, 2048).expect("wrong private key");
        let wrong_private_key_pem = wrong_private_key
            .to_pkcs8_pem(Default::default())
            .expect("wrong private key pem")
            .to_string();
        (public_key_pem, private_key_pem, wrong_private_key_pem)
    });

    TestCertificates {
        encrypt: EncryptOptions::from_public_key_pem(public_key_pem)
            .expect("encrypt options from pem")
            .with_key_id(KEY_ID.to_vec()),
        decrypt: DecryptOptions::from_private_key_pem(private_key_pem)
            .expect("decrypt options from pem")
            .with_expected_key_id(KEY_ID.to_vec()),
        wrong_decrypt: DecryptOptions::from_private_key_pem(wrong_private_key_pem)
            .expect("wrong decrypt options from pem")
            .with_expected_key_id(KEY_ID.to_vec()),
    }
}

fn write_to_buf<W: std::io::Write>(
    buf: &mut W,
    payloads: &mut [Payload],
    encrypt: &mut EncryptOptions,
) -> std::io::Result<()> {
    let mut encrypt_ctx = PayloadContext::Encrypt(encrypt);
    for payload in payloads.iter_mut() {
        payload.write_all(buf, &mut encrypt_ctx)?;
    }
    Ok(())
}

fn read_payloads(buffer: &[u8], decrypt: &mut DecryptOptions) -> std::io::Result<Vec<Payload>> {
    use std::io::{BufReader, Cursor};

    let mut reader = BufReader::new(Cursor::new(buffer));
    let mut payloads = Vec::new();
    let mut decrypt_ctx = PayloadContext::Decrypt(decrypt);
    while let Ok(header) = brec::PayloadHeader::read(&mut reader) {
        payloads.push(
            Payload::read(&mut reader, &header, &mut decrypt_ctx).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?,
        );
    }
    Ok(payloads)
}

fn read_payloads_from_buffered(
    buffer: &[u8],
    decrypt: &mut DecryptOptions,
) -> std::io::Result<(Vec<Payload>, usize)> {
    use brec::BufferedReader;
    use std::io::Cursor;

    let mut inner = Cursor::new(buffer);
    let mut reader = BufferedReader::new(&mut inner);
    let mut payloads = Vec::new();
    let mut decrypt_ctx = PayloadContext::Decrypt(decrypt);
    while let Ok(header) = brec::PayloadHeader::read(&mut reader) {
        match <Payload as TryExtractPayloadFromBuffered<Payload>>::try_read(
            &mut reader,
            &header,
            &mut decrypt_ctx,
        )
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?
        {
            ReadStatus::Success(payload) => {
                payloads.push(payload);
            }
            ReadStatus::NotEnoughData(_needed) => {
                reader.refill()?;
            }
        }
    }
    Ok((payloads, reader.consumed()))
}

use std::sync::atomic::{AtomicUsize, Ordering};

static BYTES: AtomicUsize = AtomicUsize::new(0);
static INSTANCES: AtomicUsize = AtomicUsize::new(0);

fn report(bytes: usize, instance: usize) {
    use num_format::{Locale, ToFormattedString};

    BYTES.fetch_add(bytes, Ordering::Relaxed);
    INSTANCES.fetch_add(instance, Ordering::Relaxed);
    let bytes = BYTES.load(Ordering::Relaxed);
    println!(
        "Generated {} payloads ({}, {} B)",
        INSTANCES
            .load(Ordering::Relaxed)
            .to_formatted_string(&Locale::en),
        if bytes > 1024 * 1024 {
            format!(
                "{} Mb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else if bytes > 1024 {
            format!(
                "{} Kb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else {
            format!("{} B", bytes.to_formatted_string(&Locale::en))
        },
        bytes.to_formatted_string(&Locale::en)
    );
}

fn get_proptest_config() -> ProptestConfig {
    let cases = std::env::var("BREC_STRESS_PAYLOADS_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(cases)
    }
}

fn max() -> usize {
    std::env::var("BREC_STRESS_PAYLOADS_MAX_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

proptest! {
    #![proptest_config(get_proptest_config())]

    #[test]
    fn check_sizes(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        let mut certs = certificates();
        let mut bytes = 0;
        let mut encrypt_ctx = PayloadContext::Encrypt(&mut certs.encrypt);
        let mut decrypt_ctx = PayloadContext::Decrypt(&mut certs.decrypt);
        for payload in payloads.iter_mut() {
            let mut buffer = Vec::new();
            payload.write_all(&mut buffer, &mut encrypt_ctx)?;
            let expected_size = PacketHeader::payload_size(payload, &mut encrypt_ctx)?;
            assert_eq!(buffer.len(), expected_size as usize);

            let mut cursor = std::io::Cursor::new(buffer);
            let header = brec::PayloadHeader::read(&mut cursor)?;
            let restored = Payload::read(&mut cursor, &header, &mut decrypt_ctx)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
            assert_eq!(payload, &restored);
            bytes += expected_size as usize;
        }
        report(bytes, payloads.len());
    }

    #[test]
    fn try_read_from(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        let mut certs = certificates();
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads, &mut certs.encrypt)?;
        let restored = read_payloads(&buf, &mut certs.decrypt)?;
        assert_eq!(payloads.len(), restored.len());
        for (left, right) in restored.iter().zip(payloads.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), restored.len());
    }

    #[test]
    fn try_read_from_buffered(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        let mut certs = certificates();
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads, &mut certs.encrypt)?;
        let write = buf.len() as u64;
        let (restored, read) = read_payloads_from_buffered(&buf, &mut certs.decrypt)?;
        assert_eq!(write, read as u64);
        assert_eq!(payloads.len(), restored.len());
        for (left, right) in restored.iter().zip(payloads.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), restored.len());
    }

    #[test]
    fn test_read_from(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        use std::io::Cursor;
        let mut certs = certificates();
        let mut bytes = 0;
        let mut encrypt_ctx = PayloadContext::Encrypt(&mut certs.encrypt);
        let mut decrypt_ctx = PayloadContext::Decrypt(&mut certs.decrypt);
        for payload in payloads.iter_mut() {
            let mut buf: Vec<u8> = Vec::new();
            payload.write_all(&mut buf, &mut encrypt_ctx)?;
            bytes += buf.len();
            let mut cursor = Cursor::new(buf);
            let header = brec::PayloadHeader::read(&mut cursor)?;
            match <Payload as TryExtractPayloadFrom<Payload>>::try_read(
                &mut cursor,
                &header,
                &mut decrypt_ctx,
            )? {
                ReadStatus::Success(restored) => {
                    assert_eq!(payload, &restored);
                }
                ReadStatus::NotEnoughData(_needed) => {
                    panic!("No data to read payload");
                }
            }
        }
        report(bytes, payloads.len());
    }

    #[test]
    fn test_read_from_buffered(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        use std::io::Cursor;
        let mut certs = certificates();
        let mut bytes = 0;
        let mut encrypt_ctx = PayloadContext::Encrypt(&mut certs.encrypt);
        let mut decrypt_ctx = PayloadContext::Decrypt(&mut certs.decrypt);
        for payload in payloads.iter_mut() {
            let mut buf: Vec<u8> = Vec::new();
            payload.write_all(&mut buf, &mut encrypt_ctx)?;
            bytes += buf.len();
            let mut cursor = Cursor::new(buf);
            let header = brec::PayloadHeader::read(&mut cursor)?;
            match <Payload as TryExtractPayloadFromBuffered<Payload>>::try_read(
                &mut cursor,
                &header,
                &mut decrypt_ctx,
            )? {
                ReadStatus::Success(restored) => {
                    assert_eq!(payload, &restored);
                }
                ReadStatus::NotEnoughData(_needed) => {
                    panic!("No data to read payload");
                }
            }
        }
        report(bytes, payloads.len());
    }

}

#[test]
fn decrypt_fails_with_wrong_certificate() {
    use proptest::test_runner::TestRunner;

    let mut certs = certificates();
    let payload = PayloadC::arbitrary()
        .new_tree(&mut TestRunner::default())
        .unwrap()
        .current();
    let mut encrypt_ctx = PayloadContext::Encrypt(&mut certs.encrypt);
    let encrypted =
        <PayloadC as PayloadEncode>::encode(&payload, &mut encrypt_ctx).expect("encrypt payload");
    let mut wrong_decrypt_ctx = PayloadContext::Decrypt(&mut certs.wrong_decrypt);
    let result = <PayloadC as PayloadDecode<PayloadC>>::decode(&encrypted, &mut wrong_decrypt_ctx);
    assert!(result.is_err());
}

#[test]
fn mixed_protocol_supports_open_and_encrypted_payloads() {
    let mut certs = certificates();
    let mut payloads = vec![
        Payload::PayloadA(PayloadA {
            field_u8: 1,
            field_u16: 2,
            field_u32: 3,
            field_u64: 4,
            field_u128: 5,
            field_i8: -1,
            field_i16: -2,
            field_i32: -3,
            field_i64: -4,
            field_i128: -5,
            field_f32: 1.25,
            field_f64: 2.5,
            field_bool: true,
            field_str: "open-a".to_owned(),
            vec_u8: vec![1, 2, 3],
            vec_u16: vec![4, 5],
            vec_u32: vec![6, 7],
            vec_u64: vec![8, 9],
            vec_u128: vec![10, 11],
            vec_i8: vec![-1, -2],
            vec_i16: vec![-3, -4],
            vec_i32: vec![-5, -6],
            vec_i64: vec![-7, -8],
            vec_i128: vec![-9, -10],
            vec_str: vec!["alpha".to_owned(), "beta".to_owned()],
        }),
        Payload::PayloadB(PayloadB {
            field_u8: Some(11),
            field_u16: Some(12),
            field_u32: Some(13),
            field_u64: Some(14),
            field_u128: Some(15),
            field_i8: Some(-11),
            field_i16: Some(-12),
            field_i32: Some(-13),
            field_i64: Some(-14),
            field_i128: Some(-15),
            field_f32: Some(3.25),
            field_f64: Some(6.5),
            field_bool: Some(false),
            field_str: Some("open-b".to_owned()),
            vec_u8: Some(vec![21, 22, 23]),
            vec_u16: Some(vec![24, 25]),
            vec_u32: Some(vec![26, 27]),
            vec_u64: Some(vec![28, 29]),
            vec_u128: Some(vec![30, 31]),
            vec_i8: Some(vec![-21, -22]),
            vec_i16: Some(vec![-23, -24]),
            vec_i32: Some(vec![-25, -26]),
            vec_i64: Some(vec![-27, -28]),
            vec_i128: Some(vec![-29, -30]),
            vec_str: Some(vec!["gamma".to_owned(), "delta".to_owned()]),
        }),
        Payload::PayloadC(PayloadC {
            field_u8: 31,
            field_u16: 32,
            field_u32: 33,
            field_u64: 34,
            field_u128: 35,
            field_struct_a: NestedStructCA {
                field_u8: 41,
                field_u16: 42,
                field_u32: 43,
            },
            field_struct_b: Some(NestedStructCB {
                field_i8: -41,
                field_i16: -42,
                field_i32: -43,
            }),
            field_struct_c: vec![NestedStructCC {
                field_bool: true,
                field_str: "secret-c".to_owned(),
                vec_u8: vec![51, 52, 53],
            }],
            field_enum: NestedEnumC::One("enum-secret".to_owned()),
            vec_enum: vec![NestedEnumC::Three, NestedEnumC::Two(vec![61, 62, 63])],
        }),
        Payload::PayloadD(PayloadD::String("secret-d".to_owned())),
    ];

    let mut buf = Vec::new();
    write_to_buf(&mut buf, &mut payloads, &mut certs.encrypt).expect("write mixed protocol buffer");

    let restored = read_payloads(&buf, &mut certs.decrypt).expect("read mixed protocol buffer");
    assert_eq!(restored, payloads);
}
