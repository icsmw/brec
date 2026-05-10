use crate::Error;
use crate::SourceWritable;
use std::path::Path;

/// Compile-time file name for a generated source artifact.
pub trait FileName {
    const FILE_NAME: &'static str;
}

/// TypeScript module export name represented by a generated file.
///
/// For example `blocks.ts` exports the `Block` type, so other generated files
/// can import it without hard-coding import strings in several places.
pub trait ModuleName {
    const MODULE_NAME: &'static str;
}

/// Shared metadata for a generated TypeScript module.
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

/// Generated module that can be imported with `import type`.
pub trait Importable: ModuleFile {
    fn import_statement(&self) -> String {
        format!(
            "import type {{ {} }} from \"./{}\";",
            self.module_name(),
            self.module_path()
        )
    }
}

/// Generated module that can be re-exported from `index.ts`.
pub trait Exportable: ModuleFile {
    fn export_statement(&self) -> String {
        format!("export * from \"./{}\";", self.module_path())
    }
}

impl<T: ModuleFile> Importable for T {}

impl<T: ModuleFile> Exportable for T {}

/// Module included in the public npm package barrel file.
pub trait PackageModule: Importable + Exportable {}

impl<T: Importable + Exportable> PackageModule for T {}

/// Generated file that can be written to an output directory by file name.
pub trait OutputFile: SourceWritable {
    fn file_name(&self) -> &'static str;
}

impl<T: FileName + SourceWritable> OutputFile for T {
    fn file_name(&self) -> &'static str {
        T::FILE_NAME
    }
}

/// Writes one generated output file into a package directory.
pub fn write_output_file(out: &Path, file: &dyn OutputFile) -> Result<(), Error> {
    file.write_to_path(&out.join(file.file_name()))
}
