use crate::Error;
use crate::SourceWritable;
use std::path::Path;

pub trait FileName {
    const FILE_NAME: &'static str;
}

pub trait ModuleName {
    const MODULE_NAME: &'static str;
}

pub trait ModuleFile {
    fn file_name(&self) -> &'static str;
    fn module_name(&self) -> &'static str;

    fn module_path(&self) -> &'static str {
        self.file_name()
            .strip_suffix(".ts")
            .unwrap_or(self.file_name())
    }
}

impl<T: FileName + ModuleName> ModuleFile for T {
    fn file_name(&self) -> &'static str {
        T::FILE_NAME
    }

    fn module_name(&self) -> &'static str {
        T::MODULE_NAME
    }
}

pub trait Importable: ModuleFile {
    fn import_statement(&self) -> String {
        format!(
            "import type {{ {} }} from \"./{}\";",
            self.module_name(),
            self.module_path()
        )
    }
}

pub trait Exportable: ModuleFile {
    fn export_statement(&self) -> String {
        format!("export * from \"./{}\";", self.module_path())
    }
}

impl<T: ModuleFile> Importable for T {}

impl<T: ModuleFile> Exportable for T {}

pub trait PackageModule: Importable + Exportable {}

impl<T: Importable + Exportable> PackageModule for T {}

pub trait OutputFile: SourceWritable {
    fn file_name(&self) -> &'static str;
}

impl<T: FileName + SourceWritable> OutputFile for T {
    fn file_name(&self) -> &'static str {
        T::FILE_NAME
    }
}

pub fn write_output_file(out: &Path, file: &dyn OutputFile) -> Result<(), Error> {
    file.write_to_path(&out.join(file.file_name()))
}
