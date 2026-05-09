use crate::{Error, SourceWritable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property(String);

impl Property {
    pub fn new(name: impl AsRef<str>) -> Result<Self, Error> {
        let name = name.as_ref();
        if is_valid_identifier(name) {
            Ok(Self(name.to_owned()))
        } else {
            Ok(Self(serde_json::to_string(name)?))
        }
    }
}

impl SourceWritable for Property {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        writer.write(&self.0)?;
        Ok(())
    }
}

fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}
