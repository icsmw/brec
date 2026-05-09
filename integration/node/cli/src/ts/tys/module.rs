pub trait Module {
    const FILE_NAME: &'static str;
    const MODULE_NAME: &'static str;
}

pub trait ModuleDyn {
    fn file_name(&self) -> &'static str;
    fn module_name(&self) -> &'static str;
    fn module_path(&self) -> &'static str;
}

impl<T: Module> ModuleDyn for T {
    fn file_name(&self) -> &'static str {
        T::FILE_NAME
    }

    fn module_name(&self) -> &'static str {
        T::MODULE_NAME
    }

    fn module_path(&self) -> &'static str {
        T::FILE_NAME.strip_suffix(".ts").unwrap_or(T::FILE_NAME)
    }
}

pub trait Importable: ModuleDyn {
    fn import_statement(&self) -> String {
        format!(
            "import type {{ {} }} from \"./{}\";",
            self.module_name(),
            self.module_path()
        )
    }
}

pub trait Exportable: ModuleDyn {
    fn export_statement(&self) -> String {
        format!("export * from \"./{}\";", self.module_path())
    }
}

impl<T: ModuleDyn> Importable for T {}

impl<T: ModuleDyn> Exportable for T {}

pub trait ImportableExportable: Importable + Exportable {}

impl<T: Importable + Exportable> ImportableExportable for T {}
