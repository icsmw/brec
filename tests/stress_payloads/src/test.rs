use brec::prelude::*;
use proptest::prelude::*;

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

fn write_to_buf<W: std::io::Write>(buf: &mut W, payloads: &mut [Payload]) -> std::io::Result<()> {
    for payload in payloads.iter_mut() {
        payload.write_all(buf)?;
    }
    Ok(())
}

fn read_payloads(buffer: &[u8]) -> std::io::Result<Vec<Payload>> {
    use std::io::{BufReader, Cursor};

    let mut reader = BufReader::new(Cursor::new(buffer));
    let mut payloads = Vec::new();
    while let Ok(header) = brec::PayloadHeader::read(&mut reader) {
        payloads.push(Payload::read(&mut reader, &header).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?);
    }
    Ok(payloads)
}

fn read_payloads_from_buffered(buffer: &[u8]) -> std::io::Result<(Vec<Payload>, usize)> {
    use brec::BufferedReader;
    use std::io::Cursor;

    let mut inner = Cursor::new(buffer);
    let mut reader = BufferedReader::new(&mut inner);
    let mut payloads = Vec::new();
    while let Ok(header) = brec::PayloadHeader::read(&mut reader) {
        match <Payload as TryExtractPayloadFromBuffered<Payload>>::try_read(&mut reader, &header)
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
        let mut bytes = 0;
        for payload in payloads.iter_mut() {
            let mut buffer = Vec::new();
            payload.write_all(&mut buffer)?;
            let expected_size = PacketHeader::payload_size(payload)?;
            assert_eq!(buffer.len(), expected_size as usize);
            bytes += buffer.len();
        }
        report(bytes, payloads.len());
    }

    #[test]
    fn try_read_from(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads)?;
        let restored = read_payloads(&buf)?;
        assert_eq!(payloads.len(), restored.len());
        for (left, right) in restored.iter().zip(payloads.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), restored.len());
    }

    #[test]
    fn try_read_from_buffered(mut payloads in proptest::collection::vec(any::<Payload>(), 1..max())) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads)?;
        let write = buf.len() as u64;
        let (restored, read) = read_payloads_from_buffered(&buf)?;
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
        let mut bytes = 0;
        for payload in payloads.iter_mut() {
            let mut buf: Vec<u8> = Vec::new();
            payload.write_all(&mut buf)?;
            bytes += buf.len();
            let mut cursor = Cursor::new(buf);
            let header = brec::PayloadHeader::read(&mut cursor)?;
            match <Payload as TryExtractPayloadFrom<Payload>>::try_read(&mut cursor, &header)?
            {
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
        let mut bytes = 0;
        for payload in payloads.iter_mut() {
            let mut buf: Vec<u8> = Vec::new();
            payload.write_all(&mut buf)?;
            bytes += buf.len();
            let mut cursor = Cursor::new(buf);
            let header = brec::PayloadHeader::read(&mut cursor)?;
            match <Payload as TryExtractPayloadFromBuffered<Payload>>::try_read(&mut cursor, &header)?
            {
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
