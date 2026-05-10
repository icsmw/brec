use super::deps;
use crate::*;
use std::path::PathBuf;

pub(super) struct CargoToml<'a> {
    model: &'a Model,
    dir: PathBuf,
    protocol_dir: PathBuf,
    deps: Option<PathBuf>,
}

impl<'a> CargoToml<'a> {
    pub(super) const FILE_NAME: &'static str = "Cargo.toml";

    const PACKAGE_NAME: &'static str = "bindings";
    const PACKAGE_EDITION: &'static str = "2024";
    const LIB_NAME: &'static str = "bindings";
    const LIB_CRATE_TYPES: &'static [&'static str] = &["cdylib", "rlib"];

    pub(super) fn new(
        model: &'a Model,
        dir: impl Into<PathBuf>,
        protocol_dir: impl Into<PathBuf>,
        deps: Option<PathBuf>,
    ) -> Self {
        Self {
            model,
            dir: dir.into(),
            protocol_dir: protocol_dir.into(),
            deps,
        }
    }

    pub(super) fn source_package(&self) -> &str {
        &self.model.package
    }

    fn crate_types() -> Result<String, Error> {
        Self::toml_array(Self::LIB_CRATE_TYPES)
    }

    fn toml_string(value: &str) -> Result<String, Error> {
        Ok(serde_json::to_string(value)?)
    }

    fn toml_array(values: &[&str]) -> Result<String, Error> {
        let values = values
            .iter()
            .map(|value| Self::toml_string(value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(format!("[{}]", values.join(", ")))
    }

    fn dependencies(&self) -> Result<Vec<deps::Dependency>, Error> {
        if !self.protocol_dir.is_dir() {
            return Err(Error::Cli(format!(
                "protocol path does not point to an existing directory: {}",
                self.protocol_dir.display()
            )));
        }
        let mut dependencies = vec![deps::Dependency::path(
            "protocol",
            &self.model.package,
            relative_path(&self.dir, &self.protocol_dir)?,
        )];
        dependencies.extend(deps::dependencies(&self.dir, self.deps.as_deref())?);
        Ok(dependencies)
    }
}

impl SourceWritable for CargoToml<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("[workspace]")?;
        writer.ln("")?;
        writer.ln("[package]")?;
        writer.ln(format!("name = {}", Self::toml_string(Self::PACKAGE_NAME)?))?;
        writer.ln(format!(
            "version = {}",
            Self::toml_string(&self.model.version)?
        ))?;
        writer.ln(format!(
            "edition = {}",
            Self::toml_string(Self::PACKAGE_EDITION)?
        ))?;
        writer.ln("")?;
        writer.ln("[lib]")?;
        writer.ln(format!("name = {}", Self::toml_string(Self::LIB_NAME)?))?;
        writer.ln(format!("crate-type = {}", Self::crate_types()?))?;
        writer.ln("")?;
        writer.ln("[dependencies]")?;
        for dep in self.dependencies()? {
            dep.write(writer)?;
        }
        Ok(())
    }
}
