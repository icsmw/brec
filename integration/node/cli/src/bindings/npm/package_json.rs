use crate::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) const NATIVE_BINDING_PATH: &str = "native/bindings.node";
const NPM_DEPS: &str = include_str!("../../../deps.npm.toml");

#[derive(Deserialize)]
struct NpmDeps {
    #[serde(default)]
    dependencies: BTreeMap<String, NpmDependency>,
    #[serde(rename = "devDependencies")]
    #[serde(default)]
    dev_dependencies: BTreeMap<String, NpmDependency>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum NpmDependency {
    Version(String),
    Local { path: String },
}

pub(super) struct PackageJson<'a> {
    model: &'a Model,
    dir: PathBuf,
    deps: Option<PathBuf>,
}

impl<'a> PackageJson<'a> {
    pub fn new(model: &'a Model, dir: impl Into<PathBuf>, deps: Option<&Path>) -> Self {
        Self {
            model,
            dir: dir.into(),
            deps: deps.map(Path::to_path_buf),
        }
    }

    pub fn validate_dependencies(&self) -> Result<(), Error> {
        self.dependencies().map(|_| ())
    }

    pub fn files(&self) -> Vec<String> {
        fn compiled_file(source: &str, ext: &str) -> String {
            let stem = source.strip_suffix(".ts").unwrap_or(source);
            format!("{stem}.{ext}")
        }
        let mut files = vec!["index.js".to_owned(), "index.d.ts".to_owned()];
        for source in [
            BlocksFile::FILE_NAME,
            PayloadFile::FILE_NAME,
            PacketFile::FILE_NAME,
        ] {
            files.push(compiled_file(source, "js"));
            files.push(compiled_file(source, "d.ts"));
        }
        files.push(NATIVE_BINDING_PATH.to_owned());
        files
    }
}

impl FileName for PackageJson<'_> {
    const FILE_NAME: &'static str = "package.json";
}

impl SourceWritable for PackageJson<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        let deps = self.dependencies()?;
        let dependencies = deps
            .dependencies
            .into_iter()
            .map(|(name, dep)| Ok((name, json!(dep.package_spec()?))))
            .collect::<Result<serde_json::Map<_, _>, Error>>()?;
        let dev_dependencies = deps
            .dev_dependencies
            .into_iter()
            .map(|(name, dep)| Ok((name, json!(dep.package_spec()?))))
            .collect::<Result<serde_json::Map<_, _>, Error>>()?;
        let mut package = json!({
            "name": self.model.package,
            "version": self.model.version,
            "private": true,
            "main": "index.js",
            "types": "index.d.ts",
            "scripts": {
                "build": "tsc -p tsconfig.json"
            },
            "files": self.files(),
            "devDependencies": dev_dependencies
        });
        if !dependencies.is_empty() {
            package["dependencies"] = json!(dependencies);
        }

        writer.write(format!("{}\n", serde_json::to_string_pretty(&package)?))?;
        Ok(())
    }
}

impl PackageJson<'_> {
    fn dependencies(&self) -> Result<NpmDeps, Error> {
        let mut deps = toml::from_str::<NpmDeps>(NPM_DEPS)?;
        let built_in_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        deps.resolve_paths(built_in_dir, &self.dir)?;
        if let Some(path) = &self.deps {
            let content = std::fs::read_to_string(path)?;
            let config_dir = path
                .parent()
                .ok_or_else(|| Error::MissingParent(path.to_path_buf()))?;
            let mut overrides = toml::from_str::<NpmDeps>(&content)?;
            overrides.resolve_paths(config_dir, &self.dir)?;
            deps.dependencies.extend(overrides.dependencies);
            deps.dev_dependencies.extend(overrides.dev_dependencies);
        }
        Ok(deps)
    }
}

impl NpmDeps {
    fn resolve_paths(&mut self, config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
        for dep in self.dependencies.values_mut() {
            dep.resolve_path(config_dir, output_dir)?;
        }
        for dep in self.dev_dependencies.values_mut() {
            dep.resolve_path(config_dir, output_dir)?;
        }
        Ok(())
    }
}

impl NpmDependency {
    fn resolve_path(&mut self, config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
        let Self::Local { path } = self else {
            return Ok(());
        };
        let relative = RelativePath::from_config(&path, config_dir)?;
        *path = relative.path(output_dir)?.display().to_string();
        Ok(())
    }

    fn package_spec(&self) -> Result<String, Error> {
        match self {
            Self::Version(version) => Ok(version.clone()),
            Self::Local { path } => Ok(format!("file:{path}")),
        }
    }
}
