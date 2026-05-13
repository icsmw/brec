use crate::{Error, SourceWritable};

use super::{Interface, TypeAlias};

/// Top-level TypeScript declaration emitted for a protocol type.
///
/// Named Rust structs become interfaces when they have named fields; tuple
/// structs, empty types, and enums are represented as type aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Interface(Interface),
    Type(TypeAlias),
}

impl SourceWritable for Declaration {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        match self {
            Self::Interface(interface) => interface.write(writer),
            Self::Type(alias) => alias.write(writer),
        }
    }
}
