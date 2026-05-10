use crate::*;
use brec_scheme::SchemeFile;

/// Union of payloads that can be encoded through the generated WASM package.
///
/// Context payloads are intentionally skipped because generated bindings use
/// an empty context. Built-in `Bytes` and `String` payloads are added unless
/// the protocol scheme explicitly disables default payloads.
pub struct PayloadUnion(Vec<Type>);

impl PayloadUnion {
    pub fn from_scheme(scheme: &SchemeFile) -> Result<Self, Error> {
        let mut variants = scheme
            .payloads
            .iter()
            .filter(|payload| !payload.is_ctx && payload.is_bincode)
            .map(|payload| {
                Ok(Type::object(vec![Field::required(
                    &payload.fullname,
                    Type::Named(payload.fullname.clone()),
                )?]))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if !scheme.config.no_default_payloads {
            variants.push(Type::object(vec![Field::required(
                "Bytes",
                Type::array(Type::Number),
            )?]));
            variants.push(Type::object(vec![Field::required("String", Type::String)?]));
        }

        Ok(Self(variants))
    }
}

impl SourceWritable for PayloadUnion {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        if self.0.is_empty() {
            return writer.ln("export type Payload = never;");
        }
        TypeAlias::new("Payload", Type::Union(self.0.clone())).write(writer)
    }
}
