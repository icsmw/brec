use crate::FormatterWritable;

use super::{Interface, TypeAlias};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Interface(Interface),
    Type(TypeAlias),
}

impl FormatterWritable for Declaration {
    fn write(&self, writer: &mut crate::FormatterWriter) -> fmt::Result {
        match self {
            Self::Interface(interface) => interface.write(writer),
            Self::Type(alias) => alias.write(writer),
        }
    }
}
