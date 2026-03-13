use brec::prelude::*;

#[cfg(test)]
use std::io::Cursor;

/// Demo RSA public key used only by this example.
///
/// In a real application you would usually load it from a file, secret store,
/// certificate, or service configuration.
const EXAMPLE_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQChr4I12dOaK6aEqRWOsTOg+rVg
hVMnyrkxNdYQAC32fTFv/2QeHj4cQzo2OfLmYZH57v2xzozzzb1rjPVjEhkfQMjb
zLHTLwqJLH3Qk+d8F2iEz7ex+CMojzd3IrhSldHAhDuGAJxaKjEb/484bJGkyBpg
K8Ychq0iXRt3p5Z6cQIDAQAB
-----END PUBLIC KEY-----"#;

/// Matching RSA private key for the demo public key above.
const EXAMPLE_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBAKGvgjXZ05orpoSp
FY6xM6D6tWCFUyfKuTE11hAALfZ9MW//ZB4ePhxDOjY58uZhkfnu/bHOjPPNvWuM
9WMSGR9AyNvMsdMvCoksfdCT53wXaITPt7H4IyiPN3ciuFKV0cCEO4YAnFoqMRv/
jzhskaTIGmArxhyGrSJdG3enlnpxAgMBAAECgYAEyoSbP+crTFvU1oXTAqE7BfLV
911tcm5mbOf49WhnQ3JxlSnMUq0YfU1+Sd1OwllnBJPz7uDyYIhaZYTn+KNR47Iy
gAc3qgNrmujEB8k0v6hF/gvEviCjQF4AwgnW8890HDWGReETSi2SCbgJ4H02Q0aF
qLpZq/Tjy9R+5ywdmQJBANQTYBboFIbCCeefeJDpVDL0sM9mnolfecjqYRCbjPjV
I/O0KpY2INl//W9iCuTJYIGdtXYoBffp4xDixLDUFa8CQQDDLFyR+C/c3o/V0yWV
JzbOb2Rv3tA3ev+U1YGh+/MBpVr2FJuPZUdjCJ8NZeo7LnZNw/3bUVCE7wuHPVka
vpnfAkEAhurQnaIFtPlqzbURQbd+/m/WsAtL3n8j/iLFn4gl9gO6vIao9Sj4WwZm
195aqdRHFg6b69Bog6CC+TIbCZfTNwJAQnt1/PMBusbFUBzgjHITJTaki8bmPj/T
l6sywS7FlCXzWiei5bGmI4HoS/QPWaF2Av9kFbUZLG8RCjxHgeizGQJAf5qyWJMm
NMmr271dkZaU9BRhsOndnJs+Jspe4Wj/5yj0cIkU/kU03IZBaj9AlLEMIdnf47I+
MRKDQcT+qYzk5w==
-----END PRIVATE KEY-----"#;

/// Optional identifier embedded into the crypto envelope.
///
/// It is useful when an application rotates keys and wants the reader to
/// verify that the packet was encrypted for the expected key.
const KEY_ID: &[u8] = b"examples/crypt/demo-key";

/// A small example block attached to every packet.
#[block]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    pub request_id: u32,
}

/// Packet payload used in this example.
///
/// The important part is `crypt`: the payload is still regular bincode/serde
/// data, but `brec` will now expect crypto options in `PayloadContext` and will
/// transparently encrypt/decrypt the payload body.
#[payload(bincode, crypt)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub message: String,
}

// Generates crate-local glue code:
// - `Block`
// - `Payload`
// - `Packet`
// - `PayloadContext<'a>`
// - `PacketBufReader`
// - `Reader` / `Writer`
brec::generate!();

fn main() {}

#[cfg(test)]
fn usage(
    message: &str,
    request_id: u32,
) -> Result<(Vec<u8>, GreetingPayload), Box<dyn std::error::Error>> {
    let original = GreetingPayload {
        message: message.to_owned(),
    };
    let mut packet = Packet::new(
        vec![Block::MetaBlock(MetaBlock { request_id })],
        Some(Payload::GreetingPayload(original.clone())),
    );

    // Writer side: use the public key.
    let mut encrypt =
        EncryptOptions::from_public_key_pem(EXAMPLE_PUBLIC_KEY_PEM)?.with_key_id(KEY_ID.to_vec());
    let mut encrypt_ctx = PayloadContext::Encrypt(&mut encrypt);

    let mut bytes = Vec::new();
    packet.write_all(&mut bytes, &mut encrypt_ctx)?;

    // Reader side: use the matching private key.
    let mut decrypt = DecryptOptions::from_private_key_pem(EXAMPLE_PRIVATE_KEY_PEM)?
        .with_expected_key_id(KEY_ID.to_vec());
    let mut decrypt_ctx = PayloadContext::Decrypt(&mut decrypt);

    let restored = {
        let mut source = Cursor::new(bytes.as_slice());
        let mut reader = PacketBufReader::new(&mut source);

        let packet = match reader.read(&mut decrypt_ctx)? {
            NextPacket::Found(packet) => packet,
            NextPacket::NotEnoughData(_) => {
                return Err("unexpected read status: NotEnoughData".into());
            }
            NextPacket::NoData => return Err("unexpected read status: NoData".into()),
            NextPacket::NotFound => return Err("unexpected read status: NotFound".into()),
            NextPacket::Skipped => return Err("unexpected read status: Skipped".into()),
        };

        match packet.payload {
            Some(Payload::GreetingPayload(payload)) => payload,
            _ => return Err("payload was not restored".into()),
        }
    };

    Ok((bytes, restored))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_example() -> Result<(), Box<dyn std::error::Error>> {
        let original = GreetingPayload {
            message: "hello from crypt payload".to_owned(),
        };
        let (bytes, restored) = usage(&original.message, 42)?;

        // Quick sanity check: encrypted bytes on the wire should not contain
        // the original plaintext message in plain form.
        assert!(
            !bytes
                .windows(original.message.len())
                .any(|window| window == original.message.as_bytes())
        );
        assert_eq!(restored, original);
        Ok(())
    }
}
