use crate::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const CARGO_DEPS: &str = include_str!("../../../deps.cargo.toml");

#[derive(Deserialize)]
struct CargoDeps {
    dependencies: BTreeMap<String, DependencySpec>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum DependencySpec {
    Version(String),
    Detail(DependencyDetail),
}

#[derive(Clone, Deserialize)]
pub(super) struct DependencyDetail {
    pub(super) package: Option<String>,
    pub(super) path: Option<String>,
    pub(super) version: Option<String>,
    #[serde(default)]
    pub(super) features: Vec<String>,
}

#[derive(Clone)]
pub(super) struct Dependency {
    name: String,
    spec: DependencySpec,
}

pub(super) fn dependencies(
    output_dir: &Path,
    override_path: Option<&Path>,
) -> Result<Vec<Dependency>, Error> {
    CargoDeps::load(output_dir, override_path)
}

impl CargoDeps {
    fn load(output_dir: &Path, override_path: Option<&Path>) -> Result<Vec<Dependency>, Error> {
        let mut deps = toml::from_str::<Self>(CARGO_DEPS)?;
        deps.resolve_paths(Path::new(env!("CARGO_MANIFEST_DIR")), output_dir)?;

        if let Some(path) = override_path {
            let mut overrides = Self::from_path(path)?;
            let config_dir = path
                .parent()
                .ok_or_else(|| Error::MissingParent(path.to_path_buf()))?;
            overrides.resolve_paths(config_dir, output_dir)?;
            deps.dependencies.extend(overrides.dependencies);
        }

        Ok(deps
            .dependencies
            .into_iter()
            .map(|(name, spec)| Dependency { name, spec })
            .collect())
    }

    fn from_path(path: &Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    fn resolve_paths(&mut self, config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
        for spec in self.dependencies.values_mut() {
            spec.resolve_paths(config_dir, output_dir)?;
        }
        Ok(())
    }
}

impl Dependency {
    pub(super) fn path(
        name: impl Into<String>,
        package: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Self {
        Self {
            name: name.into(),
            spec: DependencySpec::Detail(DependencyDetail {
                package: Some(package.into()),
                path: Some(path.as_ref().display().to_string()),
                version: None,
                features: Vec::new(),
            }),
        }
    }

    pub(super) fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        match &self.spec {
            DependencySpec::Version(version) => writer.ln(format!(
                "{} = {}",
                self.name,
                serde_json::to_string(version)?
            )),
            DependencySpec::Detail(detail) => {
                writer.ln(format!("{} = {{ {} }}", self.name, detail.inline_table()?))
            }
        }
    }
}

impl DependencyDetail {
    fn resolve_path(&mut self, config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let relative = RelativePath::from_config(path, config_dir)?;
        self.path = Some(relative.path(output_dir)?.display().to_string());
        Ok(())
    }

    fn inline_table(&self) -> Result<String, Error> {
        let mut fields = Vec::new();
        if let Some(package) = &self.package {
            fields.push(format!("package = {}", serde_json::to_string(package)?));
        }
        if let Some(path) = &self.path {
            fields.push(format!("path = {}", serde_json::to_string(path)?));
        }
        if let Some(version) = &self.version {
            fields.push(format!("version = {}", serde_json::to_string(version)?));
        }
        if !self.features.is_empty() {
            let features = self
                .features
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?;
            fields.push(format!("features = [{}]", features.join(", ")));
        }
        Ok(fields.join(", "))
    }
}

impl DependencySpec {
    fn resolve_paths(&mut self, config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
        match self {
            Self::Version(_) => Ok(()),
            Self::Detail(detail) => detail.resolve_path(config_dir, output_dir),
        }
    }
}
