use brec::prelude::*;
use proptest::prelude::*;

use crate::*;

brec::include_generated!("crate::*");

impl Arbitrary for Payload {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            PayloadA::arbitrary().prop_map(Payload::PayloadA),
            PayloadB::arbitrary().prop_map(Payload::PayloadB),
            PayloadC::arbitrary().prop_map(Payload::PayloadC),
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

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(100)
    })]


    #[test]
    fn try_read_from(mut payloads in proptest::collection::vec(any::<Payload>(), 1..1000)) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads)?;
        let restored = read_payloads(&buf)?;
        assert_eq!(payloads.len(), restored.len());
        for (left, right) in restored.iter().zip(payloads.iter()) {
            assert_eq!(left, right);
        }
        println!("Generated {} payloads ({} bytes)", payloads.len(), buf.len());
    }


    #[test]
    fn try_read_from_buffered(mut payloads in proptest::collection::vec(any::<Payload>(), 1..1000)) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &mut payloads)?;
        let write = buf.len() as u64;
        let (restored, read) = read_payloads_from_buffered(&buf)?;
        assert_eq!(write, read as u64);
        assert_eq!(payloads.len(), restored.len());
        for (left, right) in restored.iter().zip(payloads.iter()) {
            assert_eq!(left, right);
        }
        println!("Generated {} payloads ({} bytes)", payloads.len(), buf.len());
    }


}
