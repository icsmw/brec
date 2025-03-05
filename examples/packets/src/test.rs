use brec::prelude::*;
use proptest::prelude::*;

use crate::*;

brec::include_generated!("crate::*");

#[derive(PartialEq, PartialOrd, Debug)]
struct WrappedPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

impl From<&WrappedPacket> for Packet {
    fn from(wrapped: &WrappedPacket) -> Self {
        Packet::new(wrapped.blocks.clone(), wrapped.payload.clone())
    }
}

impl From<Packet> for WrappedPacket {
    fn from(pkg: Packet) -> Self {
        WrappedPacket {
            blocks: pkg.blocks,
            payload: pkg.payload,
        }
    }
}

impl Arbitrary for WrappedPacket {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            prop::collection::vec(Block::arbitrary(), 1..10),
            prop::option::of(Payload::arbitrary()),
        )
            .prop_map(|(blocks, payload)| WrappedPacket { blocks, payload })
            .boxed()
    }
}

#[derive(Debug)]
struct LitteredPacket {
    pub packet: WrappedPacket,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

impl Arbitrary for LitteredPacket {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            WrappedPacket::arbitrary(),
            prop::option::of(prop::collection::vec(any::<u8>(), 1..2500)),
            prop::option::of(prop::collection::vec(any::<u8>(), 1..2500)),
        )
            .prop_map(|(packet, before, after)| LitteredPacket {
                packet,
                before,
                after,
            })
            .boxed()
    }
}

fn write_to_buf<W: std::io::Write>(
    buffer: &mut W,
    packets: &[WrappedPacket],
) -> std::io::Result<()> {
    for wrapped in packets.iter() {
        let mut packet: Packet = wrapped.into();
        packet.write_all(buffer)?;
    }
    Ok(())
}

fn write_to_buf_with_litter<W: std::io::Write>(
    buffer: &mut W,
    packets: &[LitteredPacket],
) -> std::io::Result<()> {
    for wrapped in packets.iter() {
        let mut packet: Packet = (&wrapped.packet).into();
        if let Some(litter) = wrapped.before.as_ref() {
            buffer.write_all(litter)?
        }
        packet.write_all(buffer)?;
        if let Some(litter) = wrapped.after.as_ref() {
            buffer.write_all(litter)?
        }
    }
    Ok(())
}

fn read_packets(buffer: &[u8]) -> std::io::Result<Vec<Packet>> {
    use std::io::{BufReader, Cursor};

    let mut packets: Vec<Packet> = Vec::new();
    let mut inner = BufReader::new(Cursor::new(buffer));
    let mut reader: PacketBufReader<_, std::io::BufWriter<Vec<u8>>> =
        PacketBufReader::new(&mut inner);
    let mut ignored = 0;
    let mut cb = |bytes: &[u8]| {
        println!("ignored: {}", bytes.len());
    };
    // reader
    //     .add_rule(Rule::Ignored(brec::RuleFnDef::Dynamic(Box::new(cb))))
    //     .unwrap();
    loop {
        match reader.read() {
            Ok(next) => match next {
                NextPacket::Found(packet) => packets.push(packet),
                NextPacket::NotFound => {
                    // Data will be refilled with next call
                }
                NextPacket::NotEnoughData(_needed) => {
                    // Data will be refilled with next call
                }
                _ => {
                    break;
                }
            },
            Err(err) => {
                println!("ERR: {err}");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err.to_string(),
                ));
            }
        };
    }
    Ok(packets)
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
        INSTANCES.load(Ordering::Relaxed),
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

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(200)
    })]

    #[test]
    fn try_read_from(packets in proptest::collection::vec(any::<WrappedPacket>(), 1..1000)) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &packets)?;
        let restored = read_packets(&buf)?;
        let count = restored.len();
        assert_eq!(packets.len(), count);
        for (left, right) in restored.into_iter().map(|pkg|pkg.into()).collect::<Vec<WrappedPacket>>().iter().zip(packets.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), count);
    }

    #[test]
    fn try_read_with_litter(packets in proptest::collection::vec(any::<LitteredPacket>(), 1..2000)) {
        let mut buf = Vec::new();
        write_to_buf_with_litter(&mut buf, &packets)?;
        let restored = read_packets(&buf)?;
        let count = restored.len();
        let packets = packets.into_iter().map(|p| p.packet).collect::<Vec<WrappedPacket>>();
        assert_eq!(packets.len(), count);
        for (left, right) in restored.into_iter().map(|pkg|pkg.into()).collect::<Vec<WrappedPacket>>().iter().zip(packets.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), count);
    }

}
