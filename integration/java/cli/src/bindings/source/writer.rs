use crate::Error;

/// Java CLI adapter over the shared integration source writer.
///
/// The shared writer is intentionally independent from this binary's error
/// type; this wrapper preserves the existing `Result<(), Error>` API.
pub struct SourceWriter<'a> {
    inner: brec_inter_tools::SourceWriter<'a>,
}

impl<'a> SourceWriter<'a> {
    pub fn new(dest: &'a mut dyn std::fmt::Write, tab: &'a mut brec_inter_tools::Tab) -> Self {
        Self {
            inner: brec_inter_tools::SourceWriter::new(dest, tab),
        }
    }

    pub fn ln(&mut self, line: impl AsRef<str>) -> Result<(), Error> {
        self.inner.ln(line)?;
        Ok(())
    }

    pub fn write(&mut self, content: impl AsRef<str>) -> Result<(), Error> {
        self.inner.write(content)?;
        Ok(())
    }

    pub fn block(&mut self, content: impl AsRef<str>) -> Result<(), Error> {
        self.inner.block(content)?;
        Ok(())
    }

    pub fn tab(&mut self) {
        self.inner.tab();
    }

    pub fn back(&mut self) {
        self.inner.back();
    }
}
