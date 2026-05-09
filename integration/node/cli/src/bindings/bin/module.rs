use crate::*;

impl<'a> FormattableRust for ApiModule<'a> {
    fn write_rust(&self, writer: &mut FormatterWriter) -> std::fmt::Result {
        writer.ln("use napi::bindgen_prelude::{Buffer, Error, Result, Status};")?;
        writer.ln("use napi::{Env, Unknown};")?;
        writer.ln("use napi_derive::napi;")?;
        writer.ln("use protocol::{Block, Packet, Payload};")?;
        writer.ln("")?;
        writer
            .ln("fn to_napi_error(prefix: &'static str, err: impl std::fmt::Display) -> Error {")?;
        writer.tab();
        writer.ln(r#"Error::new(Status::GenericFailure, format!("{prefix}: {err}"))"#)?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        for api in &self.apis {
            api.write_rust(writer)?;
        }
        Ok(())
    }
}
