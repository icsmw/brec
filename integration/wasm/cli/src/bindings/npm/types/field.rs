use super::property::Property;
use super::ty::Type;
use crate::{Error, SourceWritable};

/// Object field inside a generated TypeScript interface or inline object type.
///
/// Field names are stored as `Property` so Rust names that are not legal
/// TypeScript identifiers are still emitted as quoted object keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: Property,
    ty: Type,
    optional: bool,
}

impl Field {
    pub fn required(name: impl AsRef<str>, ty: Type) -> Result<Self, Error> {
        Ok(Self {
            name: Property::new(name)?,
            ty,
            optional: false,
        })
    }

    pub fn optional(name: impl AsRef<str>, ty: Type) -> Result<Self, Error> {
        Ok(Self {
            name: Property::new(name)?,
            ty,
            optional: true,
        })
    }
}

impl SourceWritable for Field {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        let optional = if self.optional { "?" } else { "" };
        self.name.write(writer)?;
        writer.write(format!("{}: ", optional))?;
        self.ty.write(writer)?;
        writer.ln(";")
    }
}
