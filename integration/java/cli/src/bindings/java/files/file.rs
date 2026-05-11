use crate::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) struct JavaFile {
    pub(super) path: PathBuf,
    content: String,
}

#[derive(Clone, Copy)]
pub(super) enum JavaPackage {
    Root,
    Block,
    Payload,
}

impl JavaPackage {
    fn name(self) -> &'static str {
        match self {
            Self::Root => JAVA_PACKAGE,
            Self::Block => "com.icsmw.brec.block",
            Self::Payload => "com.icsmw.brec.payload",
        }
    }

    fn path(self, file_name: &str) -> PathBuf {
        match self {
            Self::Root => PathBuf::from(file_name),
            Self::Block => PathBuf::from("block").join(file_name),
            Self::Payload => PathBuf::from("payload").join(file_name),
        }
    }
}

impl JavaFile {
    pub(super) fn new(
        model: &Model,
        package: JavaPackage,
        file_name: impl Into<String>,
        write: impl FnOnce(&mut SourceWriter<'_>) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        let file_name = file_name.into();
        let mut content = String::new();
        let mut tab = Tab::default();
        let mut writer = SourceWriter::new(&mut content, &mut tab);
        writer.ln(format!("package {};", package.name()))?;
        writer.ln("")?;
        FileHeader::new(&file_name, model).write(&mut writer)?;
        writer.ln("")?;
        write(&mut writer)?;
        Ok(Self {
            path: package.path(&file_name),
            content,
        })
    }

    pub(super) fn write_to(&self, out: &Path) -> Result<(), Error> {
        let path = out.join(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &self.content)?;
        Ok(())
    }
}

pub(super) fn write_imports(writer: &mut SourceWriter, imports: &[&str]) -> Result<(), Error> {
    let imports = imports.iter().copied().collect::<BTreeSet<_>>();
    for import in &imports {
        writer.ln(format!("import {import};"))?;
    }
    if !imports.is_empty() {
        writer.ln("")?;
    }
    Ok(())
}
