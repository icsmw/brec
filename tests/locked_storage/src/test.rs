use crate::*;
use proptest::prelude::*;
use serial_test::serial;

pub const MATCH: &str = "-match-";

brec::generate!(payloads_derive = "Clone, Debug");

#[derive(Debug)]
pub struct WrappedPacket {
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
        any::<Record>()
            .prop_map(|record| WrappedPacket {
                blocks: vec![Block::Metadata(record.mt)],
                payload: Some(Payload::String(record.msg)),
            })
            .boxed()
    }
}

const BIN_LOG_FILE: &str = "locked_storage_test_measurements.bin";

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(1)
    })]

    #[test]
    #[serial]
    fn bin_logs(rows in proptest::collection::vec(any::<WrappedPacket>(), 10)) {
        storage::create_file(rows, 100, BIN_LOG_FILE)?;
        storage::read_file(BIN_LOG_FILE)?;
    }

}
