mod blocks;
mod payloads;

pub use blocks::*;
pub use payloads::*;

brec::generate!(scheme);

#[cfg(any(test, feature = "test-utils"))]
use proptest::arbitrary::any;
#[cfg(any(test, feature = "test-utils"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "test-utils"))]
use proptest::strategy::ValueTree;

#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug)]
pub struct WrappedPacket {
    pub blocks: Vec<Block>,
    pub payload: Option<Payload>,
}

#[cfg(any(test, feature = "test-utils"))]
impl From<WrappedPacket> for Packet {
    fn from(value: WrappedPacket) -> Self {
        Packet::new(value.blocks, value.payload)
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Arbitrary for WrappedPacket {
    type Parameters = bool;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(no_blocks: bool) -> Self::Strategy {
        if no_blocks {
            prop::option::of(Payload::arbitrary())
                .prop_map(|payload| WrappedPacket {
                    blocks: Vec::new(),
                    payload,
                })
                .boxed()
        } else {
            (
                prop::collection::vec(Block::arbitrary(), 1..20),
                prop::option::of(Payload::arbitrary()),
            )
                .prop_map(|(blocks, payload)| WrappedPacket { blocks, payload })
                .boxed()
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub fn gen_n<T: Arbitrary>(n: usize) -> Vec<T> {
    let mut runner = proptest::test_runner::TestRunner::default();
    let strat = any::<T>();

    (0..n)
        .map(|_| strat.new_tree(&mut runner).unwrap().current())
        .collect()
}

#[cfg(any(test, feature = "test-utils"))]
pub fn gen_n_packets(n: usize) -> Vec<Packet> {
    gen_n::<WrappedPacket>(n)
        .into_iter()
        .map(Into::into)
        .collect()
}
