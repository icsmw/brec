use crate::*;

/// Represents a rule callback, which can be either a dynamic closure or a static function.
///
/// This abstraction allows users to define flexible behaviors (e.g., `FnMut` closures),
/// while also supporting simple static functions for performance or clarity.
///
/// # Type Parameters
/// - `D`: Dynamic callback type (e.g., boxed `FnMut` or `Fn`)
/// - `S`: Static function type
pub enum RuleFnDef<D, S> {
    /// A dynamic, possibly stateful closure.
    Dynamic(D),

    /// A static function pointer.
    Static(S),
}

/// Callback used when `PacketBufReaderDef` encounters unrecognized data.
pub type IgnoredCallback = RuleFnDef<Box<dyn FnMut(&[u8]) + Send + 'static>, fn(&[u8])>;

/// Callback used to handle and write unrecognized data to a separate writer.
pub type WriteIgnoredCallback<W> = RuleFnDef<
    Box<dyn FnMut(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()> + Send + 'static>,
    fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>,
>;

/// Lightweight view over blocks parsed in zero-copy mode during prefiltering.
pub struct PeekedBlocksDef<'a, BR> {
    inner: &'a [BR],
}

/// Generated bridge between a concrete user block type and a referred block enum variant.
///
/// This trait is implemented by `brec::generate!()` for each user-defined block, allowing
/// typed access like `blocks.get::<MyBlock>()` without matching on `BlockReferred` manually.
pub trait PeekAs<'a, T> {
    /// Concrete block type exposed when the cast succeeds.
    type Peeked: 'a;

    /// Attempts to view the current referred block as a specific concrete block type.
    fn peek_as(&'a self) -> Option<&'a Self::Peeked>;
}

impl<'a, BR> PeekedBlocksDef<'a, BR> {
    /// Creates a lightweight borrowed view over a sequence of referred blocks.
    pub fn new(inner: &'a [BR]) -> Self {
        Self { inner }
    }

    /// Returns the number of referred blocks in the view.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` when the view contains no blocks.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the block at the given position as a `PeekedBlockDef`.
    pub fn nth(&self, index: usize) -> Option<PeekedBlockDef<'a, BR>> {
        self.inner.get(index).map(PeekedBlockDef::new)
    }

    /// Returns the first block in the view, if any.
    pub fn first(&self) -> Option<PeekedBlockDef<'a, BR>> {
        self.nth(0)
    }

    /// Returns `true` when at least one block can be viewed as the requested type.
    pub fn has<T>(&self) -> bool
    where
        BR: PeekAs<'a, T>,
    {
        self.inner.iter().any(|block| block.peek_as().is_some())
    }

    /// Returns the first block of the requested concrete type.
    pub fn get_as<T>(&self) -> Option<&'a <BR as PeekAs<'a, T>>::Peeked>
    where
        BR: PeekAs<'a, T>,
    {
        self.inner.iter().find_map(|block| block.peek_as())
    }

    /// Returns the first block of the requested concrete type.
    pub fn get<T>(&self) -> Option<&'a <BR as PeekAs<'a, T>>::Peeked>
    where
        BR: PeekAs<'a, T>,
    {
        self.get_as::<T>()
    }

    /// Finds the first block of the requested type that matches the predicate.
    pub fn find<T, F>(&self, mut predicate: F) -> Option<&'a <BR as PeekAs<'a, T>>::Peeked>
    where
        BR: PeekAs<'a, T>,
        F: FnMut(&<BR as PeekAs<'a, T>>::Peeked) -> bool,
    {
        self.inner
            .iter()
            .filter_map(|block| block.peek_as())
            .find(|block| predicate(block))
    }

    /// Iterates only over blocks of the requested concrete type.
    pub fn iter_as<T>(&self) -> impl Iterator<Item = &'a <BR as PeekAs<'a, T>>::Peeked> + 'a
    where
        BR: PeekAs<'a, T>,
    {
        self.inner.iter().filter_map(|block| block.peek_as())
    }

    /// Iterates over all blocks in their referred representation.
    pub fn iter(&self) -> impl Iterator<Item = PeekedBlockDef<'a, BR>> + 'a {
        self.inner.iter().map(PeekedBlockDef::new)
    }

    /// Advanced escape hatch for direct access to the underlying referred blocks.
    pub fn as_slice(&self) -> &'a [BR] {
        self.inner
    }
}

/// Borrowed view over a single referred block exposed during prefiltering.
pub struct PeekedBlockDef<'a, BR> {
    inner: &'a BR,
}

impl<'a, BR> PeekedBlockDef<'a, BR> {
    /// Wraps a referred block into a `PeekedBlockDef`.
    pub fn new(inner: &'a BR) -> Self {
        Self { inner }
    }

    /// Advanced escape hatch for direct access to the underlying referred block.
    pub fn as_referred(&self) -> &'a BR {
        self.inner
    }

    /// Attempts to view the current block as the requested concrete block type.
    pub fn as_type<T>(&self) -> Option<&'a <BR as PeekAs<'a, T>>::Peeked>
    where
        BR: PeekAs<'a, T>,
    {
        self.inner.peek_as()
    }
}

impl<'a, BR> std::ops::Deref for PeekedBlockDef<'a, BR> {
    type Target = BR;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, BR> IntoIterator for PeekedBlocksDef<'a, BR> {
    type Item = PeekedBlockDef<'a, BR>;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, BR>, fn(&'a BR) -> PeekedBlockDef<'a, BR>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().map(PeekedBlockDef::new)
    }
}

/// Callback to prefilter packets early by inspecting block metadata before full parsing.
pub type PrefilterCallback<BR> = RuleFnDef<
    Box<dyn Fn(PeekedBlocksDef<'_, BR>) -> bool + Send + 'static>,
    fn(PeekedBlocksDef<'_, BR>) -> bool,
>;

/// Callback to filter packets by payload content.
pub type PayloadFilterCallback =
    RuleFnDef<Box<dyn Fn(&[u8]) -> bool + Send + 'static>, fn(&[u8]) -> bool>;

/// Callback to filter fully parsed packets.
pub type FilterCallback<B, P, Inner> = RuleFnDef<
    Box<dyn Fn(&PacketDef<B, P, Inner>) -> bool + Send + 'static>,
    fn(&PacketDef<B, P, Inner>) -> bool,
>;

/// Defines processing rules used by `PacketBufReaderDef`.
///
/// These rules serve as lightweight hooks, allowing the user to handle non-`brec` binary data,
/// selectively parse packets, or filter by content without modifying the reader itself.
///
/// Rules are additive and modular. Only one rule of each type can be active at a time.
/// Discrete rule kinds supported by `RulesDef`.
///
/// `enum_ids` also generates helper identifier types for this enum. Those
/// generated items do not carry through variant docs cleanly, so
/// `missing_docs` is suppressed locally for this declaration.
#[allow(missing_docs)]
#[enum_ids::enum_ids(display)]
pub enum RuleDef<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
{
    /// Triggered when unknown or malformed data is encountered.
    Ignored(IgnoredCallback),

    /// Cheap prefilter during partial block parsing.
    ///
    /// This rule receives a lightweight block view and can reject packets before full parsing,
    /// avoiding unnecessary payload decoding and full block materialization.
    Prefilter(PrefilterCallback<BR>),

    /// Filters packets by inspecting the raw payload buffer (before decoding).
    FilterPayload(PayloadFilterCallback),

    /// Filters fully parsed packets.
    FilterPacket(FilterCallback<B, P, Inner>),
}

/// Internal container for rule management used by `PacketBufReaderDef`.
///
/// Stores registered rules and provides runtime dispatch based on rule type.
///
/// Users do not interact with `RulesDef` directly - it's driven by `PacketBufReaderDef`.
pub struct RulesDef<
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    /// Raw list of installed rules in evaluation order.
    pub rules: Vec<RuleDef<B, BR, P, Inner>>,
}

impl<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef> Default
    for RulesDef<B, BR, P, Inner>
{
    /// Initializes an empty rule set.
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}
impl<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
    RulesDef<B, BR, P, Inner>
{
    /// Adds a new rule to the rule set.
    ///
    /// Only one rule of each type is allowed at any time. Adding a duplicate
    /// will result in `Error::RuleDuplicate`.
    pub fn add_rule(&mut self, rule: RuleDef<B, BR, P, Inner>) -> Result<(), Error> {
        match &rule {
            RuleDef::Prefilter(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::Prefilter(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::FilterPayload(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::FilterPayload(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::FilterPacket(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::FilterPacket(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::Ignored(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Ignored(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
        };
        self.rules.push(rule);
        Ok(())
    }

    /// Removes a rule by its identifier (`RuleDefId`).
    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules
            .retain(|r| r.id().to_string() != rule.to_string());
    }

    /// Executes the `Ignored` rule (if defined) with the provided data.
    pub fn ignore(&mut self, buffer: &[u8]) -> Result<(), Error> {
        for rule in self.rules.iter_mut() {
            match rule {
                RuleDef::Ignored(cb) => match cb {
                    RuleFnDef::Static(cb) => cb(buffer),
                    RuleFnDef::Dynamic(cb) => cb(buffer),
                },
                _ignored => {}
            }
        }
        Ok(())
    }

    /// Runs the prefilter rule (if defined) and returns whether to continue parsing.
    pub fn prefilter(&self, blocks: &[BR]) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::Prefilter(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(PeekedBlocksDef::new(blocks)),
            RuleFnDef::Dynamic(cb) => cb(PeekedBlocksDef::new(blocks)),
        }
    }

    /// Runs the payload filter rule (if defined) and returns whether to keep the packet.
    pub fn filter_payload(&self, buffer: &[u8]) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::FilterPayload(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(buffer),
            RuleFnDef::Dynamic(cb) => cb(buffer),
        }
    }

    /// Returns `true` when a raw-payload filter rule is configured.
    pub fn has_payload_filter(&self) -> bool {
        self.rules
            .iter()
            .any(|rule| matches!(rule, RuleDef::FilterPayload(..)))
    }

    /// Runs the full packet filter rule on a parsed packet.
    pub fn filter_packet(&self, packet: &PacketDef<B, P, Inner>) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::FilterPacket(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(packet),
            RuleFnDef::Dynamic(cb) => cb(packet),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ByteBlock, DefaultPayloadContext, Error, ExtractPayloadFrom, IoSlices, PacketDef,
        PayloadDef, PayloadHeader, PayloadSchema, ReadBlockFrom, ReadBlockFromSlice, ReadFrom,
        ReadStatus, RuleDef, RuleDefId, RuleFnDef, RulesDef, TryExtractPayloadFrom,
        TryExtractPayloadFromBuffered, TryReadFrom, TryReadFromBuffered, WriteMutTo, WriteTo,
        WriteVectoredMutTo, WriteVectoredTo, packet::rules::PeekAs,
    };
    use std::io::Cursor;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Clone)]
    struct RuleBlock {
        field: u8,
    }

    impl RuleBlock {
        fn new(field: u8) -> Self {
            Self { field }
        }
    }

    impl crate::Size for RuleBlock {
        fn size(&self) -> u64 {
            self.field as u64
        }
    }

    impl WriteTo for RuleBlock {
        fn write<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<usize> {
            Err(std::io::Error::other("rules test block write stub"))
        }

        fn write_all<T: std::io::Write>(&self, _: &mut T) -> std::io::Result<()> {
            Err(std::io::Error::other("rules test block write_all stub"))
        }
    }

    impl WriteVectoredTo for RuleBlock {
        fn slices(&self) -> std::io::Result<IoSlices<'_>> {
            Err(std::io::Error::other("rules test block slices stub"))
        }
    }

    // These methods are required by `BlockDef` bounds in packet/rules tests.
    // They are explicit stubs and are covered by a dedicated test below.
    impl TryReadFromBuffered for RuleBlock {
        fn try_read<T: std::io::BufRead>(_: &mut T) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }

    impl TryReadFrom for RuleBlock {
        fn try_read<T: std::io::Read + std::io::Seek>(
            _: &mut T,
        ) -> Result<ReadStatus<Self>, Error> {
            Err(Error::Test)
        }
    }

    impl ReadFrom for RuleBlock {
        fn read<T: std::io::Read>(_: &mut T) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }

    impl ReadBlockFrom for RuleBlock {
        fn read<T: std::io::Read>(_: &mut T, _: bool) -> Result<Self, Error> {
            Err(Error::Test)
        }
    }

    impl ReadBlockFromSlice for RuleBlock {
        fn read_from_slice<'a>(_: &'a [u8], _: bool) -> Result<Self, Error>
        where
            Self: 'a + Sized,
        {
            Err(Error::Test)
        }
    }

    impl crate::BlockDef for RuleBlock {}
    impl crate::BlockReferredDef<RuleBlock> for RuleBlock {}

    #[derive(Clone)]
    struct RulePayload {
        _field: u8,
    }

    impl RulePayload {
        fn new(field: u8) -> Self {
            Self { _field: field }
        }
    }

    impl PayloadSchema for RulePayload {
        type Context<'a> = DefaultPayloadContext;
    }

    impl WriteVectoredMutTo for RulePayload {
        fn slices(&mut self, _: &mut Self::Context<'_>) -> std::io::Result<IoSlices<'_>> {
            Err(std::io::Error::other("rules test payload slices stub"))
        }
    }

    impl WriteMutTo for RulePayload {
        fn write<T: std::io::Write>(
            &mut self,
            _: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<usize> {
            Err(std::io::Error::other("rules test payload write stub"))
        }

        fn write_all<T: std::io::Write>(
            &mut self,
            _: &mut T,
            _: &mut Self::Context<'_>,
        ) -> std::io::Result<()> {
            Err(std::io::Error::other("rules test payload write_all stub"))
        }
    }

    impl crate::PayloadSignature for RulePayload {
        fn sig(&self) -> ByteBlock {
            ByteBlock::Len4(*b"RULS")
        }
    }

    impl crate::PayloadEncodeReferred for RulePayload {
        fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
            Err(std::io::Error::other(
                "rules test payload encode_referred stub",
            ))
        }
    }

    impl crate::PayloadHooks for RulePayload {
        fn after_decode(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other(
                "rules test payload after_decode stub",
            ))
        }

        fn before_encode(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other(
                "rules test payload before_encode stub",
            ))
        }
    }

    impl crate::PayloadEncode for RulePayload {
        fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
            Err(std::io::Error::other("rules test payload encode stub"))
        }
    }

    impl crate::PayloadCrc for RulePayload {
        fn crc(&self, _: &mut Self::Context<'_>) -> std::io::Result<ByteBlock> {
            Err(std::io::Error::other("rules test payload crc stub"))
        }

        fn crc_size() -> usize {
            0
        }
    }

    impl crate::PayloadSize for RulePayload {
        fn size(&self, _: &mut Self::Context<'_>) -> std::io::Result<u64> {
            Err(std::io::Error::other("rules test payload size stub"))
        }
    }

    impl crate::PayloadInnerDef for RulePayload {}

    impl TryExtractPayloadFromBuffered<RulePayload> for RulePayload {
        fn try_read<B: std::io::BufRead>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <RulePayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<RulePayload>, Error> {
            Err(Error::Test)
        }
    }

    impl TryExtractPayloadFrom<RulePayload> for RulePayload {
        fn try_read<B: std::io::Read + std::io::Seek>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <RulePayload as PayloadSchema>::Context<'_>,
        ) -> Result<ReadStatus<RulePayload>, Error> {
            Err(Error::Test)
        }
    }

    impl ExtractPayloadFrom<RulePayload> for RulePayload {
        fn read<B: std::io::Read>(
            _: &mut B,
            _: &PayloadHeader,
            _: &mut <RulePayload as PayloadSchema>::Context<'_>,
        ) -> Result<RulePayload, Error> {
            Err(Error::Test)
        }
    }

    impl PayloadDef<RulePayload> for RulePayload {}

    struct BrA;
    struct BrB;

    enum Referred {
        A(BrA),
        B(BrB),
    }

    impl PeekAs<'_, BrA> for Referred {
        type Peeked = BrA;
        fn peek_as(&self) -> Option<&Self::Peeked> {
            match self {
                Referred::A(v) => Some(v),
                Referred::B(_) => None,
            }
        }
    }
    impl PeekAs<'_, BrB> for Referred {
        type Peeked = BrB;
        fn peek_as(&self) -> Option<&Self::Peeked> {
            match self {
                Referred::A(_) => None,
                Referred::B(v) => Some(v),
            }
        }
    }

    #[test]
    fn peeked_blocks_helpers_work() {
        let arr = vec![Referred::A(BrA), Referred::B(BrB)];
        let peeked = crate::PeekedBlocksDef::new(&arr);

        assert_eq!(peeked.len(), 2);
        assert!(!peeked.is_empty());
        assert!(peeked.has::<BrA>());
        assert!(peeked.has::<BrB>());
        assert!(peeked.get::<BrA>().is_some());
        assert!(peeked.get::<BrB>().is_some());
        assert!(peeked.find::<BrB, _>(|_| true).is_some());
        assert_eq!(peeked.iter_as::<BrA>().count(), 1);
        assert_eq!(peeked.iter_as::<BrB>().count(), 1);
        assert_eq!(peeked.iter().count(), 2);
        assert_eq!(peeked.as_slice().len(), 2);

        let first = peeked.first().expect("first");
        assert!(first.as_type::<BrA>().is_some());
        assert!(first.as_type::<BrB>().is_none());
    }

    #[test]
    fn peeked_block_accessors_and_into_iter_work() {
        let arr = vec![Referred::A(BrA), Referred::B(BrB)];
        let peeked = crate::PeekedBlocksDef::new(&arr);
        let first = peeked.first().expect("first");

        assert!(std::ptr::eq(first.as_referred(), &*first));

        let collected = peeked.into_iter().count();
        assert_eq!(collected, 2);
    }

    #[test]
    fn rules_add_duplicate_and_remove_behaviour() {
        let mut rules = RulesDef::<RuleBlock, RuleBlock, RulePayload, RulePayload>::default();

        rules
            .add_rule(RuleDef::Ignored(RuleFnDef::Static(|_| {})))
            .expect("first ignored");
        assert!(matches!(
            rules.add_rule(RuleDef::Ignored(RuleFnDef::Static(|_| {}))),
            Err(Error::RuleDuplicate)
        ));

        rules
            .add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| true)))
            .expect("first prefilter");
        assert!(matches!(
            rules.add_rule(RuleDef::Prefilter(RuleFnDef::Static(|_| true))),
            Err(Error::RuleDuplicate)
        ));

        rules.remove_rule(RuleDefId::Ignored);
        rules
            .add_rule(RuleDef::Ignored(RuleFnDef::Static(|_| {})))
            .expect("ignored can be added again after remove");

        rules
            .add_rule(RuleDef::FilterPayload(RuleFnDef::Static(|_| true)))
            .expect("first payload filter");
        assert!(matches!(
            rules.add_rule(RuleDef::FilterPayload(RuleFnDef::Static(|_| true))),
            Err(Error::RuleDuplicate)
        ));

        rules
            .add_rule(RuleDef::FilterPacket(RuleFnDef::Static(|_| true)))
            .expect("first packet filter");
        assert!(matches!(
            rules.add_rule(RuleDef::FilterPacket(RuleFnDef::Static(|_| true))),
            Err(Error::RuleDuplicate)
        ));
    }

    #[test]
    fn rules_callbacks_filter_and_ignore_paths() {
        let mut rules = RulesDef::<RuleBlock, RuleBlock, RulePayload, RulePayload>::default();

        let ignored_calls = Arc::new(AtomicUsize::new(0));
        let ignored_calls_c = ignored_calls.clone();
        rules
            .add_rule(RuleDef::Ignored(RuleFnDef::Dynamic(Box::new(move |_| {
                ignored_calls_c.fetch_add(1, Ordering::SeqCst);
            }))))
            .expect("ignored rule");

        rules
            .add_rule(RuleDef::Prefilter(RuleFnDef::Static(|blocks| {
                blocks.len() == 1
            })))
            .expect("prefilter rule");
        rules
            .add_rule(RuleDef::FilterPayload(RuleFnDef::Static(|payload| {
                payload == [1, 2, 3]
            })))
            .expect("payload rule");
        rules
            .add_rule(RuleDef::FilterPacket(RuleFnDef::Static(|packet| {
                packet.payload.is_none()
            })))
            .expect("packet rule");

        rules.ignore(&[9, 9]).expect("ignore callback");
        assert_eq!(ignored_calls.load(Ordering::SeqCst), 1);

        let blocks_a = vec![RuleBlock::new(1)];
        assert!(rules.prefilter(&blocks_a));
        assert!(!rules.prefilter(&[]));

        assert!(rules.has_payload_filter());
        assert!(rules.filter_payload(&[1, 2, 3]));
        assert!(!rules.filter_payload(&[7, 8]));

        let packet_no_payload = PacketDef::<RuleBlock, RulePayload, RulePayload>::default();
        let packet_with_payload = PacketDef::<RuleBlock, RulePayload, RulePayload>::new(
            vec![],
            Some(RulePayload::new(1)),
        );
        assert!(rules.filter_packet(&packet_no_payload));
        assert!(!rules.filter_packet(&packet_with_payload));
    }

    #[test]
    fn rules_ignore_static_callback_path_is_called() {
        static IGNORED_STATIC_CALLS: AtomicUsize = AtomicUsize::new(0);
        fn ignored_static_cb(_: &[u8]) {
            IGNORED_STATIC_CALLS.fetch_add(1, Ordering::SeqCst);
        }

        IGNORED_STATIC_CALLS.store(0, Ordering::SeqCst);
        let mut rules = RulesDef::<RuleBlock, RuleBlock, RulePayload, RulePayload>::default();
        rules
            .add_rule(RuleDef::Ignored(RuleFnDef::Static(ignored_static_cb)))
            .expect("ignored static rule");

        rules.ignore(&[1, 2, 3]).expect("ignore static callback");
        assert_eq!(IGNORED_STATIC_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn trait_required_stub_methods_return_explicit_errors() {
        let mut buffered = Cursor::new(Vec::<u8>::new());
        let mut stream = Cursor::new(Vec::<u8>::new());
        let payload_header = PayloadHeader {
            sig: ByteBlock::Len4([0, 0, 0, 0]),
            crc: ByteBlock::Len4([0, 0, 0, 0]),
            len: 0,
        };

        let block = RuleBlock::new(1);
        assert_eq!(<RuleBlock as crate::Size>::size(&block), 1);
        assert!(block.write(&mut Vec::new()).is_err());
        assert!(block.write_all(&mut Vec::new()).is_err());
        assert!(block.slices().is_err());
        assert!(matches!(
            <RuleBlock as TryReadFromBuffered>::try_read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RuleBlock as TryReadFrom>::try_read(&mut stream),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RuleBlock as ReadFrom>::read(&mut buffered),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RuleBlock as ReadBlockFrom>::read(&mut buffered, false),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RuleBlock as ReadBlockFromSlice>::read_from_slice(&[], false),
            Err(Error::Test)
        ));

        let mut payload = RulePayload::new(1);
        assert_eq!(
            <RulePayload as crate::PayloadSignature>::sig(&payload).as_slice(),
            b"RULS"
        );
        assert!(<RulePayload as crate::PayloadHooks>::before_encode(&mut payload).is_err());
        assert!(<RulePayload as crate::PayloadHooks>::after_decode(&mut payload).is_err());
        assert!(<RulePayload as crate::PayloadEncode>::encode(&payload, &mut ()).is_err());
        assert!(<RulePayload as crate::PayloadEncodeReferred>::encode(&payload, &mut ()).is_err());
        assert!(<RulePayload as crate::PayloadCrc>::crc(&payload, &mut ()).is_err());
        assert!(<RulePayload as crate::PayloadSize>::size(&payload, &mut ()).is_err());
        assert!(payload.write(&mut Vec::new(), &mut ()).is_err());
        assert!(payload.write_all(&mut Vec::new(), &mut ()).is_err());
        assert!(payload.slices(&mut ()).is_err());
        assert!(matches!(
            <RulePayload as TryExtractPayloadFromBuffered<RulePayload>>::try_read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RulePayload as TryExtractPayloadFrom<RulePayload>>::try_read(
                &mut stream,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
        assert!(matches!(
            <RulePayload as ExtractPayloadFrom<RulePayload>>::read(
                &mut buffered,
                &payload_header,
                &mut ()
            ),
            Err(Error::Test)
        ));
    }
}
