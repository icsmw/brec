use crate::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const CARGO_DEPS: &str = include_str!("../../../deps.cargo.toml");

#[derive(Deserialize)]
struct CargoDeps {
    dependencies: BTreeMap<String, DependencySpec>,
}

/// Cargo dependency value supported by `deps.cargo.toml`.
///
/// The compact string form is used for normal registry versions. The detailed
/// form supports package renames, local paths, versions, and feature lists.
#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum DependencySpec {
    Version(String),
    Detail(DependencyDetail),
}

/// Detailed Cargo dependency declaration rendered as an inline TOML table.
#[derive(Clone, Deserialize)]
pub(super) struct DependencyDetail {
    pub(super) package: Option<String>,
    pub(super) path: Option<String>,
    pub(super) version: Option<String>,
    #[serde(default)]
    pub(super) features: Vec<String>,
}

/// Named Cargo dependency ready to be written into the generated manifest.
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
    /// Loads built-in dependency defaults and applies optional user overrides.
    ///
    /// Local path dependencies are resolved relative to the TOML file where
    /// they were declared, then rewritten relative to the generated crate.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn writes_version_dependency() {
        let dep = Dependency {
            name: "serde".to_owned(),
            spec: DependencySpec::Version("1".to_owned()),
        };
        let output = write_dependency(&dep).expect("write");

        assert_eq!(output, "serde = \"1\"\n");
    }

    #[test]
    fn writes_detail_dependency_with_features() {
        let dep = Dependency {
            name: "brec".to_owned(),
            spec: DependencySpec::Detail(DependencyDetail {
                package: None,
                path: None,
                version: Some("0.3".to_owned()),
                features: vec!["bincode".to_owned(), "java".to_owned()],
            }),
        };
        let output = write_dependency(&dep).expect("write");

        assert_eq!(
            output,
            "brec = { version = \"0.3\", features = [\"bincode\", \"java\"] }\n"
        );
    }

    #[test]
    fn resolves_detail_path_relative_to_generated_crate() {
        let root = unique_temp_dir("cargo-dep");
        let config_dir = root.join("config");
        let output_dir = root.join("generated").join("bindings");
        let package_dir = root.join("packages").join("protocol");
        fs::create_dir_all(&config_dir).expect("config dir");
        fs::create_dir_all(&output_dir).expect("output dir");
        fs::create_dir_all(&package_dir).expect("package dir");

        let mut detail = DependencyDetail {
            package: Some("protocol".to_owned()),
            path: Some("../packages/protocol".to_owned()),
            version: None,
            features: Vec::new(),
        };
        detail
            .resolve_path(&config_dir, &output_dir)
            .expect("resolve");

        assert_eq!(detail.path.as_deref(), Some("../../packages/protocol"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "brec-java-cli-{name}-{}-{nanos}",
            std::process::id(),
        ))
    }

    fn write_dependency(dep: &Dependency) -> Result<String, Error> {
        let mut output = String::new();
        let mut tab = Tab::default();
        let mut writer = SourceWriter::new(&mut output, &mut tab);
        dep.write(&mut writer)?;
        Ok(output)
    }
}
