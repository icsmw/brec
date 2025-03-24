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

const TEXT_LOG_FILE: &str = "test_measurements.log";
const JSON_LOG_FILE: &str = "test_measurements.json";
const BIN_LOG_FILE: &str = "test_measurements.bin";
const BIN_STREAM_LOG_FILE: &str = "test_measurements_stream.bin";

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(10)
    })]


    #[test]
    #[serial]
    fn text_logs(rows in proptest::collection::vec(any::<TextualRow>(), 100)) {
        tests::text::create_file(rows,10_000, TEXT_LOG_FILE)?;
        tests::text::read_file(TEXT_LOG_FILE)?;
        tests::text::filter_file(TEXT_LOG_FILE)?;
    }

    #[test]
    #[serial]
    fn json_logs(rows in proptest::collection::vec(any::<JSONRow>(), 100)) {
        tests::json::create_file(rows,10_000, JSON_LOG_FILE)?;
        tests::json::read_file(JSON_LOG_FILE)?;
        tests::json::filter_file(JSON_LOG_FILE)?;
    }

    #[test]
    #[serial]
    fn bin_logs(rows in proptest::collection::vec(any::<WrappedPacket>(), 100)) {
        tests::storage::create_file(rows, 10_000, BIN_LOG_FILE)?;
        tests::storage::read_file(BIN_LOG_FILE)?;
        tests::storage::filter_file(BIN_LOG_FILE)?;
    }

    #[test]
    #[serial]
    fn bin_logs_stream(rows in proptest::collection::vec(any::<WrappedPacket>(), 100)) {
        tests::stream::create_file(rows, 10_000, BIN_STREAM_LOG_FILE)?;
        tests::stream::read_file(BIN_STREAM_LOG_FILE)?;
        tests::stream::filter_file(BIN_STREAM_LOG_FILE)?;
    }

    #[test]
    #[serial]
    fn bin_logs_storage_streamed(rows in proptest::collection::vec(any::<WrappedPacket>(), 100)) {
        tests::streamed_storage::create_file(rows, 10_000, BIN_LOG_FILE)?;
        tests::streamed_storage::read_file(BIN_LOG_FILE)?;
        tests::streamed_storage::filter_file(BIN_LOG_FILE)?;
    }


}
