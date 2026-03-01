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
    type Peeked: 'a;

    fn peek_as(&'a self) -> Option<&'a Self::Peeked>;
}

impl<'a, BR> PeekedBlocksDef<'a, BR> {
    pub fn new(inner: &'a [BR]) -> Self {
        Self { inner }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn nth(&self, index: usize) -> Option<PeekedBlockDef<'a, BR>> {
        self.inner.get(index).map(PeekedBlockDef::new)
    }

    pub fn first(&self) -> Option<PeekedBlockDef<'a, BR>> {
        self.nth(0)
    }

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

    pub fn iter(&self) -> impl Iterator<Item = PeekedBlockDef<'a, BR>> + 'a {
        self.inner.iter().map(PeekedBlockDef::new)
    }

    /// Advanced escape hatch for direct access to the underlying referred blocks.
    pub fn as_slice(&self) -> &'a [BR] {
        self.inner
    }
}

pub struct PeekedBlockDef<'a, BR> {
    inner: &'a BR,
}

impl<'a, BR> PeekedBlockDef<'a, BR> {
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
/// Users do not interact with `RulesDef` directly — it's driven by `PacketBufReaderDef`.
pub struct RulesDef<
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
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
                if self.rules.iter().any(|r| matches!(r, RuleDef::FilterPacket(..))) {
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
