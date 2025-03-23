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
pub type IgnoredCallback = RuleFnDef<Box<dyn FnMut(&[u8])>, fn(&[u8])>;

/// Callback used to handle and write unrecognized data to a separate writer.
pub type WriteIgnoredCallback<W> = RuleFnDef<
    Box<dyn FnMut(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>>,
    fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>,
>;

/// Callback to filter packets early by inspecting block metadata before full parsing.
pub type BlocksFilterCallback<BR> = RuleFnDef<Box<dyn Fn(&[BR]) -> bool>, fn(&[BR]) -> bool>;

/// Callback to filter packets by payload content.
pub type PayloadFilterCallback = RuleFnDef<Box<dyn Fn(&[u8]) -> bool>, fn(&[u8]) -> bool>;

/// Callback to filter fully parsed packets.
pub type FilterCallback<B, P, Inner> =
    RuleFnDef<Box<dyn Fn(&PacketDef<B, P, Inner>) -> bool>, fn(&PacketDef<B, P, Inner>) -> bool>;

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

    /// Filters packets early, during partial block parsing.
    ///
    /// This rule receives a list of block references and can reject packets before full parsing,
    /// avoiding unnecessary processing.
    FilterByBlocks(BlocksFilterCallback<BR>),

    /// Filters packets by inspecting the raw payload buffer (before decoding).
    FilterByPayload(PayloadFilterCallback),

    /// Filters fully parsed packets.
    Filter(FilterCallback<B, P, Inner>),
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
            RuleDef::FilterByBlocks(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::FilterByBlocks(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::FilterByPayload(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::FilterByPayload(..)))
                {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::Filter(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Filter(..))) {
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

    /// Runs the `FilterByBlocks` rule (if defined) and returns whether to continue parsing.
    pub fn filter_by_blocks(&self, blocks: &[BR]) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::FilterByBlocks(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(blocks),
            RuleFnDef::Dynamic(cb) => cb(blocks),
        }
    }

    /// Runs the `FilterByPayload` rule (if defined) and returns whether to keep the packet.
    pub fn filter_by_payload(&self, buffer: &[u8]) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::FilterByPayload(cb) = r {
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

    /// Runs the full `Filter` rule on a parsed packet.
    pub fn filter(&self, packet: &PacketDef<B, P, Inner>) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::Filter(cb) = r {
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
