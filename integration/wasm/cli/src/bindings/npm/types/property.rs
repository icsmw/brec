use crate::{Error, SourceWritable};

/// TypeScript object property name.
///
/// Valid identifiers are emitted directly (`field: T`); every other name is
/// JSON-quoted (`"field-name": T`) so generated declarations remain valid TS.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::write_to_string;

    #[test]
    fn writes_identifier_as_is() {
        let property = Property::new("field_1").expect("property");

        assert_eq!(write_to_string(&property).expect("write"), "field_1");
    }

    #[test]
    fn quotes_non_identifier_property_name() {
        let property = Property::new("field-name").expect("property");

        assert_eq!(write_to_string(&property).expect("write"), "\"field-name\"");
    }

    #[test]
    fn rejects_empty_identifier_shape() {
        assert!(!is_valid_identifier(""));
    }
}
