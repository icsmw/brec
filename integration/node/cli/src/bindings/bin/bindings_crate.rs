use super::cargo_toml::CargoToml;
use super::lib_rs::BindingsLibFile;
use crate::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generated Rust crate that exposes protocol encode/decode functions to Node.
///
/// The crate is written next to the scheme by default, depends on the user's
/// protocol crate, and builds a native `bindings.node`-compatible cdylib.
pub struct BindingsCrate<'a> {
    dir: PathBuf,
    cargo_toml: CargoToml<'a>,
    model: &'a Model,
}

impl<'a> BindingsCrate<'a> {
    pub const DIR_NAME: &'static str = "bindings";

    pub fn new(
        dir: impl Into<PathBuf>,
        model: &'a Model,
        protocol_dir: impl Into<PathBuf>,
        deps: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let dir = dir.into();
        Ok(Self {
            cargo_toml: CargoToml::new(model, &dir, protocol_dir, deps),
            dir,
            model,
        })
    }

    pub fn write(&self) -> Result<(), Error> {
        let src = self.dir.join("src");
        std::fs::create_dir_all(&src)?;
        self.cargo_toml
            .write_to_path(&self.dir.join(CargoToml::FILE_NAME))?;
        self.write_lib_rs(&src.join(BindingsLibFile::FILE_NAME))
    }

    /// Builds the generated crate and returns the produced native library.
    pub fn build_release(&self) -> Result<PathBuf, Error> {
        let manifest = self.dir.join(CargoToml::FILE_NAME);
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&manifest)
            .status()?;

        if !status.success() {
            return Err(Error::CommandFailed {
                command: format!(
                    "cargo build --release --manifest-path {}",
                    manifest.display()
                ),
                status,
            });
        }

        self.find_release_artifact()
    }

    fn write_lib_rs(&self, path: &Path) -> Result<(), Error> {
        let module = ApiFile::<BindingsLibFile>::new(
            self.model,
            vec![
                Box::new(ApiBlock),
                Box::new(ApiPayload),
                Box::new(ApiPacket),
            ],
            Vec::new(),
        );
        module.write_to_path(path)
    }

    fn find_release_artifact(&self) -> Result<PathBuf, Error> {
        let release = self.target_dir().join("release");
        let artifact = release.join(Self::artifact_name());

        artifact
            .is_file()
            .then_some(artifact)
            .ok_or(Error::BindingArtifactNotFound(release))
    }

    fn target_dir(&self) -> PathBuf {
        match std::env::var_os("CARGO_TARGET_DIR") {
            Some(path) => PathBuf::from(path),
            None => self.dir.join("target"),
        }
    }

    fn artifact_name() -> String {
        format!(
            "{}{}{}",
            std::env::consts::DLL_PREFIX,
            Self::DIR_NAME,
            std::env::consts::DLL_SUFFIX
        )
    }
}
