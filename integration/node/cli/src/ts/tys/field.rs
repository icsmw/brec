use super::property::Property;
use super::ty::Type;
use crate::{Error, FormatterWritable};
use std::fmt;

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

impl FormatterWritable for Field {
    fn write(&self, writer: &mut crate::FormatterWriter) -> fmt::Result {
        let optional = if self.optional { "?" } else { "" };
        self.name.write(writer)?;
        writer.write(format!("{}: ", optional))?;
        self.ty.write(writer)?;
        writer.ln(";")
    }
}
