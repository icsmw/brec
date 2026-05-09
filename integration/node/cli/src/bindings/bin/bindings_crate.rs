use super::cargo_toml::CargoToml;
use super::lib_rs::BindingsLibFile;
use crate::*;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BindingsCrate<'a> {
    dir: PathBuf,
    cargo_toml: CargoToml<'a>,
}

impl<'a> BindingsCrate<'a> {
    pub fn new(
        dir: impl Into<PathBuf>,
        model: &'a Model,
        protocol_dir: impl Into<PathBuf>,
    ) -> Result<Self, Error> {
        Ok(Self {
            dir: dir.into(),
            cargo_toml: CargoToml::new(model, protocol_dir)?,
        })
    }

    pub fn write(&self) -> Result<(), Error> {
        let src = self.dir.join("src");
        std::fs::create_dir_all(&src)?;
        self.cargo_toml
            .write_to_path(&self.dir.join(CargoToml::FILE_NAME))?;
        self.write_lib_rs(&src.join(BindingsLibFile::FILE_NAME))
    }

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
            self.cargo_toml.source_package(),
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
        let release = self.dir.join("target").join("release");
        let candidates = [
            release.join(format!(
                "{}bindings{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            )),
            release.join("bindings.dll"),
        ];

        candidates
            .into_iter()
            .find(|path| path.is_file())
            .ok_or(Error::BindingArtifactNotFound(release))
    }
}
