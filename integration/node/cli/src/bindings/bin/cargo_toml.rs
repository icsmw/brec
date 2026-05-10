use crate::*;
use std::path::{Path, PathBuf};

pub(super) struct CargoToml<'a> {
    model: &'a Model,
    protocol_dir: PathBuf,
    brec_dir: PathBuf,
}

struct Dependency<'a> {
    name: &'static str,
    package: Option<&'a str>,
    path: Option<String>,
    version: Option<&'static str>,
    features: &'static [&'static str],
}

struct RegistryDependency {
    name: &'static str,
    version: &'static str,
    features: &'static [&'static str],
}

impl<'a> CargoToml<'a> {
    pub(super) const FILE_NAME: &'static str = "Cargo.toml";

    const PACKAGE_NAME: &'static str = "bindings";
    const PACKAGE_EDITION: &'static str = "2024";
    const LIB_NAME: &'static str = "bindings";
    const LIB_CRATE_TYPES: &'static [&'static str] = &["cdylib", "rlib"];
    const BREC_FEATURES: &'static [&'static str] = &["bincode"];
    const DEPS: &'static [RegistryDependency] = &[
        RegistryDependency {
            name: "napi",
            version: "3.8",
            features: &["serde-json", "napi6"],
        },
        RegistryDependency {
            name: "napi-derive",
            version: "3.5",
            features: &[],
        },
    ];

    pub(super) fn new(model: &'a Model, protocol_dir: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Self {
            model,
            protocol_dir: protocol_dir.into(),
            brec_dir: Self::brec_core_dir()?,
        })
    }

    pub(super) fn source_package(&self) -> &str {
        &self.model.package
    }

    fn crate_types() -> Result<String, Error> {
        Self::toml_array(Self::LIB_CRATE_TYPES)
    }

    fn brec_core_dir() -> Result<PathBuf, Error> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| Error::MissingParent(manifest_dir.clone()))?;
        Ok(root.join("lib").join("core"))
    }

    fn toml_path(path: &Path) -> Result<String, Error> {
        Self::toml_string(&path.display().to_string())
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

    fn dependencies(&self) -> Result<Vec<Dependency<'_>>, Error> {
        let mut deps = vec![
            Dependency::path("protocol", &self.protocol_dir)?.with_package(&self.model.package),
            Dependency::path("brec", &self.brec_dir)?.with_features(Self::BREC_FEATURES),
        ];
        for dep in Self::DEPS {
            deps.push(dep.dependency());
        }
        Ok(deps)
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

impl<'a> Dependency<'a> {
    fn path(name: &'static str, path: &Path) -> Result<Self, Error> {
        Ok(Self {
            name,
            package: None,
            path: Some(CargoToml::toml_path(path)?),
            version: None,
            features: &[],
        })
    }

    fn with_package(mut self, package: &'a str) -> Self {
        self.package = Some(package);
        self
    }

    fn with_features(mut self, features: &'static [&'static str]) -> Self {
        self.features = features;
        self
    }

    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        if let Some(version) = self.plain_version() {
            return writer.ln(format!(
                "{} = {}",
                self.name,
                CargoToml::toml_string(version)?
            ));
        }

        let inline = self.inline_table()?;
        writer.ln(format!("{} = {{ {} }}", self.name, inline.join(", ")))
    }

    fn plain_version(&self) -> Option<&'static str> {
        if self.package.is_none() && self.path.is_none() && self.features.is_empty() {
            return self.version;
        }
        None
    }

    fn inline_table(&self) -> Result<Vec<String>, Error> {
        let mut fields = Vec::new();
        if let Some(package) = &self.package {
            fields.push(format!("package = {}", CargoToml::toml_string(package)?));
        }
        if let Some(path) = &self.path {
            fields.push(format!("path = {path}"));
        }
        if let Some(version) = self.version {
            fields.push(format!("version = {}", CargoToml::toml_string(version)?));
        }
        if !self.features.is_empty() {
            fields.push(format!(
                "features = {}",
                CargoToml::toml_array(self.features)?
            ));
        }

        Ok(fields)
    }
}

impl RegistryDependency {
    fn dependency<'a>(&self) -> Dependency<'a> {
        Dependency {
            name: self.name,
            package: None,
            path: None,
            version: Some(self.version),
            features: self.features,
        }
    }
}
