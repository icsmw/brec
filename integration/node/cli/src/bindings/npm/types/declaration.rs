use crate::{Error, SourceWritable};

use super::{Interface, TypeAlias};

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
