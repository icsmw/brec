use crate::*;

/// Defines a callback function type used in the reading rules of `PacketBufReaderDef`.
/// The callback can be specified either as a static function or a dynamic closure,
/// providing flexibility to the user while not affecting the internal operation of `PacketBufReaderDef`.
pub enum RuleFnDef<D, S> {
    Dynamic(D),
    Static(S),
}

/// Callback type for the `Ignored` rule. For more details on rules, see `RuleDef`.
pub type IgnoredCallback = RuleFnDef<Box<dyn Fn(&[u8])>, fn(&[u8])>;

/// Callback type for the `WriteIgnored` rule. For more details on rules, see `RuleDef`.
pub type WriteIgnoredCallback<W> = RuleFnDef<
    Box<dyn Fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>>,
    fn(&mut std::io::BufWriter<W>, &[u8]) -> std::io::Result<()>,
>;

/// Callback type for the `Filter` rule. For more details on rules, see `RuleDef`.
pub type FilterCallback<B, BR, P, Inner> = RuleFnDef<
    Box<dyn Fn(&PacketReferred<B, BR, P, Inner>) -> bool>,
    fn(&PacketReferred<B, BR, P, Inner>) -> bool,
>;

/// Callback type for the `Map` rule. For more details on rules, see `RuleDef`.
pub type MapCallback<W, B, BR, P, Inner> = RuleFnDef<
    Box<
        dyn Fn(&mut std::io::BufWriter<W>, &PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>,
    >,
    fn(&mut std::io::BufWriter<W>, &PacketReferred<B, BR, P, Inner>) -> std::io::Result<()>,
>;

/// Defines rules for processing data read by `PacketBufReaderDef`. These rules function similarly to hooks.
///
/// Since `PacketBufReaderDef` is designed to read not only pure `brec` packet streams but also arbitrary data,
/// rules allow processing non-`brec` data instead of discarding it.
///
/// For example, the `Ignored` rule allows capturing data that is read but not recognized as `brec` packets.
/// Similarly, the `WriteIgnored` rule enables saving such data elsewhere.
#[enum_ids::enum_ids(display)]
pub enum RuleDef<
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    /// Called when `PacketBufReaderDef` encounters data that is not recognized as a `brec` packet.
    /// The callback receives a slice of the unrecognized data.
    Ignored(IgnoredCallback),

    /// Similar to `Ignored`, but also provides a `BufWriter` instance to allow writing unrecognized data.
    WriteIgnored(std::io::BufWriter<W>, WriteIgnoredCallback<W>),

    /// The `Filter` rule is invoked each time `PacketBufReaderDef` detects a `brec` packet.
    ///
    /// This rule is unique because it is executed during packet parsing rather than after completion.
    /// The callback receives only the `Blocks` section of the packet, which is not yet fully parsed
    /// and contains references to raw byte slices.
    ///
    /// This allows users to decide whether to continue parsing the payload and completing block parsing
    /// or to ignore the packet entirely.
    ///
    /// Using `Filter` can significantly improve performance when users are interested only in specific packet categories.
    Filter(FilterCallback<B, BR, P, Inner>),

    /// This rule is invoked for every successfully parsed `brec` packet, allowing users to perform
    /// additional manipulations before the packet is returned by `PacketBufReaderDef`.
    Map(std::io::BufWriter<W>, MapCallback<W, B, BR, P, Inner>),
}

/// Internal structure responsible for storing and managing rules.
/// This is used within `PacketBufReaderDef`, and direct access by users is not required.
pub struct RulesDef<
    W: std::io::Write,
    B: BlockDef,
    BR: BlockReferredDef<B>,
    P: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    pub rules: Vec<RuleDef<W, B, BR, P, Inner>>,
}

impl<
        W: std::io::Write,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > Default for RulesDef<W, B, BR, P, Inner>
{
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}
impl<
        W: std::io::Write,
        B: BlockDef,
        BR: BlockReferredDef<B>,
        P: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    > RulesDef<W, B, BR, P, Inner>
{
    pub fn add_rule(&mut self, rule: RuleDef<W, B, BR, P, Inner>) -> Result<(), Error> {
        match &rule {
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
            RuleDef::Map(..) => {
                if self.rules.iter().any(|r| matches!(r, RuleDef::Map(..))) {
                    return Err(Error::RuleDuplicate);
                }
            }
            RuleDef::WriteIgnored(..) => {
                if self
                    .rules
                    .iter()
                    .any(|r| matches!(r, RuleDef::WriteIgnored(..)))
                {
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
                RuleDef::WriteIgnored(dest, cb) => match cb {
                    RuleFnDef::Static(cb) => {
                        cb(dest, buffer)?;
                    }
                    RuleFnDef::Dynamic(cb) => {
                        cb(dest, buffer)?;
                    }
                },
                _ignored => {}
            }
        }
        Ok(())
    }
    pub fn filter(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> bool {
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
            RuleFnDef::Static(cb) => cb(referred),
            RuleFnDef::Dynamic(cb) => cb(referred),
        }
    }
    pub fn map(&mut self, referred: &PacketReferred<B, BR, P, Inner>) -> Result<(), Error> {
        let Some((writer, cb)) = self.rules.iter_mut().find_map(|r| {
            if let RuleDef::Map(writer, cb) = r {
                Some((writer, cb))
            } else {
                None
            }
        }) else {
            return Ok(());
        };
        match cb {
            RuleFnDef::Static(cb) => cb(writer, referred)?,
            RuleFnDef::Dynamic(cb) => cb(writer, referred)?,
        }
        Ok(())
    }
}
