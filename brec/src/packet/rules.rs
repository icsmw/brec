use crate::*;

/// Defines a callback function type used in the reading rules of `PacketBufReaderDef`.
/// The callback can be specified either as a static function or a dynamic closure,
/// providing flexibility to the user while not affecting the internal operation of `PacketBufReaderDef`.
pub enum RuleFnDef<D, S> {
    Dynamic(D),
    Static(S),
}

/// Callback type for the `Ignored` rule. For more details on rules, see `RuleDef`.
pub type IgnoredCallback = RuleFnDef<Box<dyn FnMut(&[u8])>, fn(&[u8])>;

/// Callback type for the `WriteIgnored` rule. For more details on rules, see `RuleDef`.
pub type WriteIgnoredCallback<W> = RuleFnDef<
    Box<dyn FnMut(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>>,
    fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>,
>;

/// Callback type for the `PreFilter` rule. For more details on rules, see `RuleDef`.
pub type PreFilterCallback<B, BR, P, Inner> = RuleFnDef<
    Box<dyn Fn(&PacketReferred<B, BR, P, Inner>) -> bool>,
    fn(&PacketReferred<B, BR, P, Inner>) -> bool,
>;

/// Callback type for the `Filter` rule. For more details on rules, see `RuleDef`.
pub type FilterCallback<B, P, Inner> =
    RuleFnDef<Box<dyn Fn(&PacketDef<B, P, Inner>) -> bool>, fn(&PacketDef<B, P, Inner>) -> bool>;

/// Callback type for the `EAch` rule. For more details on rules, see `RuleDef`.
pub type EachCallback<B, BR, P, Inner> = RuleFnDef<
    Box<dyn FnMut(&PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>>,
    fn(&PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>,
>;

/// Defines rules for processing data read by `PacketBufReaderDef`. These rules function similarly to hooks.
///
/// Since `PacketBufReaderDef` is designed to read not only pure `brec` packet streams but also arbitrary data,
/// rules allow processing non-`brec` data instead of discarding it.
///
/// For example, the `Ignored` rule allows capturing data that is read but not recognized as `brec` packets.
/// Similarly, the `WriteIgnored` rule enables saving such data elsewhere.
#[enum_ids::enum_ids(display)]
pub enum RuleDef<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
{
    /// Called when `PacketBufReaderDef` encounters data that is not recognized as a `brec` packet.
    /// The callback receives a slice of the unrecognized data.
    Ignored(IgnoredCallback),

    /// The `PreFilter` rule is invoked each time `PacketBufReaderDef` detects a `brec` packet.
    ///
    /// This rule is unique because it is executed during packet parsing rather than after completion.
    /// The callback receives only the `Blocks` section of the packet, which is not yet fully parsed
    /// and contains references to raw byte slices.
    ///
    /// This allows users to decide whether to continue parsing the payload and completing block parsing
    /// or to ignore the packet entirely.
    ///
    /// Using `PreFilter` can significantly improve performance when users are interested only in specific packet categories.
    PreFilter(PreFilterCallback<B, BR, P, Inner>),

    Filter(FilterCallback<B, P, Inner>),

    /// This rule is invoked for every successfully parsed `brec` packet, allowing users to perform
    /// additional manipulations before the packet is returned by `PacketBufReaderDef`.
    Each(EachCallback<B, BR, P, Inner>),
}

/// Internal structure responsible for storing and managing rules.
/// This is used within `PacketBufReaderDef`, and direct access by users is not required.
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
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}
impl<B: BlockDef, BR: BlockReferredDef<B>, P: PayloadDef<Inner>, Inner: PayloadInnerDef>
    RulesDef<B, BR, P, Inner>
{
    pub fn add_rule(&mut self, rule: RuleDef<B, BR, P, Inner>) -> Result<(), Error> {
        match &rule {
            RuleDef::PreFilter(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::PreFilter(..)))
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
            RuleDef::Each(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Each(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
        };
        self.rules.push(rule);
        Ok(())
    }

    pub fn remove_rule(&mut self, rule: RuleDefId) {
        self.rules
            .retain(|r| r.id().to_string() != rule.to_string());
    }

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
    pub fn pre_filter(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> bool {
        let Some(cb) = self.rules.iter().find_map(|r| {
            if let RuleDef::PreFilter(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return true;
        };
        match cb {
            RuleFnDef::Static(cb) => cb(referred),
            RuleFnDef::Dynamic(cb) => cb(referred),
        }
    }
    pub fn filter(&mut self, packet: &PacketDef<B, P, Inner>) -> bool {
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
    pub fn each(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> Result<(), Error> {
        let Some(cb) = self.rules.iter_mut().find_map(|r| {
            if let RuleDef::Each(cb) = r {
                Some(cb)
            } else {
                None
            }
        }) else {
            return Ok(());
        };
        match cb {
            RuleFnDef::Static(cb) => cb(referred)?,
            RuleFnDef::Dynamic(cb) => cb(referred)?,
        }
        Ok(())
    }
}
