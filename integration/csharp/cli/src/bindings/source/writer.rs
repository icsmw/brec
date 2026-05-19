use crate::*;
use std::fmt;
use std::path::Path;

/// Small indentation-aware source writer used by every generated file.
///
/// The generator builds code directly instead of using templates because most
/// files are assembled from protocol-specific model objects.
pub struct SourceWriter<'a> {
    dest: &'a mut dyn fmt::Write,
    tab: &'a mut Tab,
    line_start: bool,
}

impl<'a> SourceWriter<'a> {
    pub fn new(dest: &'a mut dyn fmt::Write, tab: &'a mut Tab) -> Self {
        Self {
            dest,
            tab,
            line_start: true,
        }
    }

    fn write_indent_if_needed(&mut self) -> Result<(), Error> {
        if self.line_start {
            write!(self.dest, "{}", self.tab)?;
            self.line_start = false;
        }
        Ok(())
    }

    pub fn ln(&mut self, line: impl AsRef<str>) -> Result<(), Error> {
        self.write(line)?;
        writeln!(self.dest)?;
        self.line_start = true;
        Ok(())
    }

    pub fn write(&mut self, content: impl AsRef<str>) -> Result<(), Error> {
        self.write_indent_if_needed()?;
        write!(self.dest, "{}", content.as_ref())?;
        Ok(())
    }

    pub fn tab(&mut self) {
        self.tab.inc();
    }

    pub fn back(&mut self) {
        self.tab.dec();
    }
}

/// Object that can render itself into a generated source file.
pub trait SourceWritable {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error>;

    fn write_to_path(&self, path: &Path) -> Result<(), Error> {
        let mut content = String::new();
        let mut tab = Tab::default();
        let mut writer = SourceWriter::new(&mut content, &mut tab);
        self.write(&mut writer)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
pub fn write_to_string(module: &dyn SourceWritable) -> Result<String, Error> {
    let mut content = String::new();
    let mut tab = Tab::default();
    let mut writer = SourceWriter::new(&mut content, &mut tab);
    module.write(&mut writer)?;
    Ok(content)
}
