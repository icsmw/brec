use crate::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

/// Writer for the generated package manifest.
///
/// It owns package metadata, dependency resolution, and the `files` whitelist
/// used by npm consumers. Local dependency overrides are resolved before
/// writing so generated examples can run from Docker build contexts.
pub(super) struct PackageJson<'a> {
    model: &'a Model,
    dir: PathBuf,
    deps: Option<PathBuf>,
    target: WasmTarget,
}

impl<'a> PackageJson<'a> {
    pub fn new(
        model: &'a Model,
        dir: impl Into<PathBuf>,
        deps: Option<&Path>,
        target: WasmTarget,
    ) -> Self {
        Self {
            model,
            dir: dir.into(),
            deps: deps.map(Path::to_path_buf),
            target,
        }
    }

    pub fn validate_dependencies(&self) -> Result<(), Error> {
        self.dependencies().map(|_| ())
    }

    /// Files produced by TypeScript compilation plus wasm-pack artifacts.
    ///
    /// The list is reused by `NpmPackage` to clean stale generated files before
    /// writing a new package.
    pub fn files(&self) -> Vec<String> {
        fn compiled_file(source: &str, ext: &str) -> String {
            let stem = source.strip_suffix(".ts").unwrap_or(source);
            format!("{stem}.{ext}")
        }
        let mut files = vec![
            "index.js".to_owned(),
            "index.d.ts".to_owned(),
            "wasmjs.js".to_owned(),
            "wasmjs.d.ts".to_owned(),
            "wasmjs_bg.wasm".to_owned(),
            "wasmjs_bg.wasm.d.ts".to_owned(),
        ];
        for source in [
            BlocksFile::FILE_NAME,
            PayloadFile::FILE_NAME,
            PacketFile::FILE_NAME,
        ] {
            files.push(compiled_file(source, "js"));
            files.push(compiled_file(source, "d.ts"));
        }
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
        if self.target == WasmTarget::Browser {
            package["type"] = json!("module");
            package["module"] = json!("index.js");
        }

        writer.write(format!("{}\n", serde_json::to_string_pretty(&package)?))?;
        Ok(())
    }
}

impl PackageJson<'_> {
    /// Loads built-in npm dependencies and applies optional user overrides.
    ///
    /// Local `path` dependencies become `file:` package specs relative to the
    /// generated package directory, which makes the package portable inside the
    /// e2e Docker build context.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn renders_version_dependency_as_plain_package_spec() {
        let dep = NpmDependency::Version("^1.0.0".to_owned());

        assert_eq!(dep.package_spec().expect("spec"), "^1.0.0");
    }

    #[test]
    fn resolves_local_dependency_relative_to_output_dir() {
        let root = unique_temp_dir("npm-dep");
        let config_dir = root.join("config");
        let output_dir = root.join("generated").join("npm");
        let package_dir = root.join("packages").join("runtime");
        fs::create_dir_all(&config_dir).expect("config dir");
        fs::create_dir_all(&output_dir).expect("output dir");
        fs::create_dir_all(&package_dir).expect("package dir");

        let mut dep = NpmDependency::Local {
            path: "../packages/runtime".to_owned(),
        };
        dep.resolve_path(&config_dir, &output_dir)
            .expect("resolve path");

        assert_eq!(
            dep.package_spec().expect("spec"),
            "file:../../packages/runtime"
        );

        fs::remove_dir_all(root).expect("cleanup");
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "brec-wasm-cli-{name}-{}-{nanos}",
            std::process::id(),
        ))
    }
}
