use crate::*;
use std::path::Path;

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
