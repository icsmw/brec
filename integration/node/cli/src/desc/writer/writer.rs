use crate::*;
use std::fmt;

pub struct FormatterWriter<'a> {
    dest: &'a mut dyn fmt::Write,
    tab: &'a mut Tab,
    line_start: bool,
}

impl<'a> FormatterWriter<'a> {
    pub fn new(dest: &'a mut dyn fmt::Write, tab: &'a mut Tab) -> Self {
        Self {
            dest,
            tab,
            line_start: true,
        }
    }

    fn write_indent_if_needed(&mut self) -> fmt::Result {
        if self.line_start {
            write!(self.dest, "{}", self.tab)?;
            self.line_start = false;
        }
        Ok(())
    }

    pub fn ln(&mut self, line: impl AsRef<str>) -> fmt::Result {
        self.write(line)?;
        writeln!(self.dest)?;
        self.line_start = true;
        Ok(())
    }

    pub fn write(&mut self, content: impl AsRef<str>) -> fmt::Result {
        self.write_indent_if_needed()?;
        write!(self.dest, "{}", content.as_ref())
    }

    pub fn tab(&mut self) {
        self.tab.inc();
    }

    pub fn back(&mut self) {
        self.tab.dec();
    }
}

pub trait FormatterWritable {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result;
}
