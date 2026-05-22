use super::Tab;
use std::fmt;

/// Small indentation-aware source writer used by generated integration files.
///
/// `tab()` and `back()` control the global indentation offset. Literal tab
/// characters in written content add relative indentation and are rendered as
/// spaces according to the writer tab settings.
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
        for ch in content.as_ref().chars() {
            match ch {
                '\n' => {
                    writeln!(self.dest)?;
                    self.line_start = true;
                }
                '\t' => {
                    self.write_indent_if_needed()?;
                    write!(self.dest, "{}", self.tab.spaces(1))?;
                }
                ch => {
                    self.write_indent_if_needed()?;
                    self.dest.write_char(ch)?;
                }
            }
        }
        Ok(())
    }

    pub fn block(&mut self, content: impl AsRef<str>) -> fmt::Result {
        let content = content.as_ref();
        let content = content.strip_prefix('\n').unwrap_or(content);
        let content = content.strip_suffix('\n').unwrap_or(content);
        self.write(content)?;
        if !content.is_empty() && !self.line_start {
            writeln!(self.dest)?;
            self.line_start = true;
        }
        Ok(())
    }

    pub fn tab(&mut self) {
        self.tab.inc();
    }

    pub fn back(&mut self) {
        self.tab.dec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_global_and_relative_indents() {
        let mut output = String::new();
        let mut tab = Tab::default();
        let mut writer = SourceWriter::new(&mut output, &mut tab);

        writer.ln("root {").unwrap();
        writer.tab();
        writer
            .block(
                r#"
field
nested {
	value
}
"#,
            )
            .unwrap();
        writer.back();
        writer.ln("}").unwrap();

        assert_eq!(
            output,
            "root {\n    field\n    nested {\n        value\n    }\n}\n"
        );
    }

    #[test]
    fn keeps_blank_lines_empty() {
        let mut output = String::new();
        let mut tab = Tab::default();
        let mut writer = SourceWriter::new(&mut output, &mut tab);

        writer.tab();
        writer.block("first\n\n\tsecond").unwrap();

        assert_eq!(output, "    first\n\n        second\n");
    }
}
